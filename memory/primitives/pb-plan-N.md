# Primitive Batch Plan: PB-N — Subtype/Color-Filtered Attack & Death Triggers

**Generated**: 2026-04-12
**Primitive**: Add `filter: Option<TargetFilter>` to two existing DSL `TriggerCondition` variants — `WheneverCreatureYouControlAttacks` and `WheneverCreatureDies` — and wire that filter through enrichment, dispatch, and hash. No new enum variants. No new types. Mirrors the already-shipped pattern on `WheneverCreatureYouControlDealsCombatDamageToPlayer { filter }`.
**CR Rules**: 508.1m (declare attackers as event point), 603.2 (triggered abilities), 603.10a (zone-change LKI for death triggers), 603.4 (intervening-if), 700.4 / 704.5g (death event), 613.1d/f (layer-resolved characteristics for filter checks).
**Dependencies**: none. `TargetFilter`, `matches_filter`, `combat_damage_filter` precedent, `DeathTriggerFilter`, hash schema for `TriggeredAbilityDef.combat_damage_filter` — all exist and ship today.
**Deferred items from prior PBs**: none directly carried; `EnchantFilter`/`TargetFilter` divergence (PB-Q4-M01) is unrelated.

---

## Summary

**Confirmed yield (post 60% discount)**: 11 cards in scope (4 attack + 7 death), 6 deferred for compound blockers documented below, 16 dropped from the original 33 because their oracle text is not actually "subtype-filtered creature-you-control attack/death". Original brief estimate ~20 was on the high end of plausible; this plan ships ~11 with zero silent skips. Yield is solid for one PB and the dispatch surface is among the smallest of any recent PB (two field additions, two dispatch sites, one hash field).

**Dispatch unification verdict**: **PASS-AS-FIELD-ADDITION**. The two trigger sites (attack-side `collect_triggers_for_event` branch in `abilities.rs:5804-5845`; death-side inline loop in `abilities.rs:4117-4214`) are structurally different functions but semantically converge on "apply a `TargetFilter` to the triggering creature against layer-resolved characteristics". Adding the same `filter` field to both DSL variants and reusing `matches_filter` at both dispatch sites is a literal copy-paste of the existing `combat_damage_filter` pattern. **No new enum variant. No `TriggerCondition::SubtypeFilteredEvent` umbrella.** A single field addition on each variant beats a new wrapper variant on every clarity axis.

**Mandatory test count**: **8 mandatory + 2 optional**. Numbered in the Test Plan section.

**Deferred-card list (6)**:
1. **Pashalik Mons** — "Pashalik Mons or another Goblin you control" — compound *self-OR-filtered* trigger; needs an `include_self` flag on the existing `DeathTriggerFilter::exclude_self` semantics. ~1-line follow-up; not in scope.
2. **Miara, Thorn of the Glade** — same self-OR-filtered shape as Pashalik.
3. **Omnath, Locus of Rage** — same self-OR-filtered shape AND blocked separately on PB-L (Landfall).
4. **Najeela, the Blade-Blossom** — "Whenever a Warrior attacks" with NO `you control` restriction. Needs a new dispatch path that fans out the trigger to ALL `WheneverCreatureYouControlAttacks` sources whose controller matches the *attacker's* controller — the existing `AnyCreatureYouControlAttacks` event already does that, but the trigger source must be on the attacker's controller's battlefield, which Najeela isn't required to be. Different dispatch shape; defer.
5. **Athreos, God of Passage** — "creature you OWN dies" — owner filter, not controller. `DeathTriggerFilter` has no `owner_you` flag; mid-game ownership flips (Donate, control-magic) make this distinct from `controller_you`. ~1-line follow-up.
6. **Skullclamp** — "equipped creature dies" — needs equipment-LKI plumbing on death triggers (the dying creature must be the one this Equipment was attached to pre-death). Out of scope, large.

---

## CR Rule Text (relevant excerpts)

### CR 508.1m (declare attackers — trigger fan-out)
> The active player taps the chosen creatures. … Any abilities that trigger on creatures attacking trigger.

### CR 603.2 (triggered abilities — event matching)
> Whenever a game event or game state matches a triggered ability's trigger event, that ability automatically triggers. … The ability doesn't do anything at this point.

### CR 603.10a (zone-change LKI / "look back in time")
> Some zone-change triggers look back in time to determine if they trigger. The list of such triggers is as follows: 
> - "When/Whenever … leaves the battlefield" abilities.
> - "When/Whenever … is put into a graveyard from the battlefield" (and "dies") abilities.
> 
> The game must check what the object would have looked like immediately before it left the battlefield.

This is the critical rule for death-side filtering: the dying creature's **pre-death subtypes/colors/types** are the values matched against the trigger filter, NOT whatever the graveyard object looks like after `move_object_to_zone` resets controller and disassembles attachments.

### CR 603.4 (intervening-if)
> If a triggered ability's trigger condition is met but its source is no longer in the appropriate zone … then it doesn't trigger. … "Intervening if" clauses are checked twice: as the ability would trigger and as it would resolve.

Filter checks here are NOT intervening-if (they are part of the trigger condition itself per CR 603.2), so they only run at trigger-collection time. No re-check on resolution.

### CR 613.1d / 613.1f (layer-resolved chars)
Subtype, color, and card-type filter reads MUST go through `calculate_characteristics()` — never raw `obj.characteristics.subtypes`. The death-side site must do this on the **graveyard object** which preserves pre-death characteristics from `move_object_to_zone`'s LKI snapshot. The attack-side site already does this (line 5823-5827).

---

## Engine Architecture Study Notes

### DSL layer (`crates/engine/src/cards/card_definition.rs`)

`TriggerCondition` enum at line 2396. The two affected variants:

- **Line 2433** `WheneverCreatureDies { controller: Option<TargetController>, exclude_self: bool, nontoken_only: bool }` — currently no filter field.
- **Line 2556** `WheneverCreatureYouControlAttacks` — currently a unit variant, no fields at all.
- **Reference precedent** (line 2563): `WheneverCreatureYouControlDealsCombatDamageToPlayer { filter: Option<TargetFilter> }` — already does exactly what we want, has been hashed and dispatched since the existing combat damage filter ship.

### Runtime layer (`crates/engine/src/state/game_object.rs`)

`TriggerEvent` enum at line 301 — runtime form (after `enrich_spec_from_def` translation). The two events that matter:

- **Line 417** `AnyCreatureDies` — fired from `CreatureDied` event handler.
- **Line 424** `AnyCreatureYouControlAttacks` — fired per attacker from `AttackersDeclared` handler.

`TriggeredAbilityDef` at line 568 carries:
- `etb_filter: Option<ETBTriggerFilter>` (l. 585)
- `death_filter: Option<DeathTriggerFilter>` (l. 590)
- `combat_damage_filter: Option<TargetFilter>` (l. 596) ← reuse this exact field for the new filter.

**Decision**: instead of adding a fourth filter field, we **reuse `combat_damage_filter`** by renaming it to `creature_filter` (or adding a new alias). However, renaming a public field creates churn across 4 card defs already using it, scripts, and hash. Cheaper and cleaner: **add one new field `attack_or_death_filter: Option<TargetFilter>`** alongside the existing three. The runtime knows which dispatch path it is in, so the field is unambiguous in context. (See "Engine Changes" below for the resolved choice.)

### Dispatch sites (`crates/engine/src/rules/abilities.rs`)

#### Attack-side (line 5800-5845, in `collect_triggers_for_event`)
Already does filter dispatch for `AnyCreatureYouControlAttacks` and `AnyCreatureYouControlDealsCombatDamageToPlayer` together (line 5809-5813). The filter check at 5821-5836 reads `combat_damage_filter` and applies `matches_filter` against `calculate_characteristics(state, attacking_id)`. **The exact code we need already runs for attacks — it's just keyed on the wrong field name.** PB-N either (a) repurposes `combat_damage_filter` for both events or (b) adds a sibling `attack_filter` field. Choice resolved below.

#### Death-side (line 4117-4214, inline in `CreatureDied` event handler — NOT in `collect_triggers_for_event`)
This dispatch site is hand-rolled (because the dying creature's pre-death controller is needed and isn't reachable through the standard `entering_object` plumbing). The site at line 4131-4213:
- Iterates all battlefield permanents (line 4137-4142)
- For each, calls `calculate_characteristics(state, obj_id)` (line 4147-4149)
- For each `AnyCreatureDies` trigger def (line 4150-4153), applies `death_filter` (line 4158-4174)
- Applies intervening-if (line 4176-4181)
- Pushes the `PendingTrigger`

**Gap**: the existing `death_filter: DeathTriggerFilter` only has `controller_you / controller_opponent / exclude_self / nontoken_only`. There is no subtype/color/keyword/type filter. PB-N adds a filter check here that mirrors line 5821-5836 but reads against the **graveyard object's characteristics** (which preserve pre-death LKI per `move_object_to_zone`).

**Critical LKI test gate**: the graveyard object MUST have its pre-death subtypes preserved by `move_object_to_zone` for this filter to work. If `move_object_to_zone` strips subtypes (it shouldn't — that would already break Skullclamp and similar), PB-N collapses. Verified: `move_object_to_zone` preserves the full Characteristics on the graveyard object exactly so the existing `AuraFellOff`/`SelfDies` look-back triggers work. Same machinery, no change needed.

### Hash dispatch (`crates/engine/src/state/hash.rs`)

`HashInto for TriggeredAbilityDef` at line 2246-2258 already hashes `combat_damage_filter`. PB-N must add the new field to this impl.

`HashInto for TriggerCondition` (DSL layer) is in the same file — must add hash arms for the new fields on `WheneverCreatureDies` and `WheneverCreatureYouControlAttacks`. Sentinel is currently 3 (post PB-Q). **Hash version bump policy: BUMP to 4** because the wire format of two existing variants gains a new tagged field. (Stop-and-flag if this is wrong — see Risks section.)

### Card-def enrichment (`crates/engine/src/cards/abilities_enrich.rs` or similar)

`enrich_spec_from_def` translates `TriggerCondition::WheneverCreatureYouControlAttacks` → `TriggeredAbilityDef { trigger_on: TriggerEvent::AnyCreatureYouControlAttacks, ... }` and `WheneverCreatureDies { ... }` → `{ trigger_on: AnyCreatureDies, death_filter: Some(DeathTriggerFilter { ... }), ... }`. Both translation arms must be extended to copy the new `filter` field into `TriggeredAbilityDef.attack_or_death_filter`.

### Match-site exhaustivity audit

`TriggerCondition` and `TriggerEvent` are matched in:

| File | Match expression | Action |
|------|------------------|--------|
| `crates/engine/src/cards/card_definition.rs` (~l. 2396) | enum definition | Add `filter: Option<TargetFilter>` field to two variants |
| `crates/engine/src/cards/abilities_enrich.rs` (or wherever `enrich_spec_from_def` lives — verify path during impl) | translation match | Two match arms extended to copy `filter` through |
| `crates/engine/src/state/hash.rs` (~l. 2246, plus the `TriggerCondition` impl) | `HashInto` impls | Hash new field on both DSL variants AND on `TriggeredAbilityDef`'s new field |
| `crates/engine/src/rules/abilities.rs` line 4150-4174 | death dispatch | Add filter check after `death_filter` block |
| `crates/engine/src/rules/abilities.rs` line 5800-5845 | attack/damage dispatch | Either rename or add sibling check (resolved below) |

**Two more files to verify** during impl (NOT planning):
- `crates/engine/src/testing/replay_harness.rs` — likely matches on `TriggerCondition` for translation; if it uses `..` rest pattern, no change needed.
- `crates/engine/src/state/serde_compat.rs` (if it exists) — version migration if hash sentinel bumps.

The runner MUST `cargo build --workspace` after the dispatch changes — exhaustive match holes here are the #1 PB compile-error source per the gotchas-infra notes on TUI/replay-viewer.

---

## Engine Changes

### Resolved design choice: add a single new field `triggering_creature_filter: Option<TargetFilter>` on `TriggeredAbilityDef`

Rationale:
- **Don't rename `combat_damage_filter`**: 4 card defs use it, renaming creates a needless diff. Existing semantics correct for damage path.
- **Don't reuse `combat_damage_filter` for attack/death**: same field name in two unrelated trigger contexts is misleading and breaks `gotchas-infra.md` "field name = dispatch context" convention.
- **Don't introduce a new TriggerCondition wrapper variant**: the gate above failed only the unification *as a single variant*, not the unification *as a single primitive*. A field that two existing variants share is the literal smallest dispatch unification.
- **Single new field name**: `triggering_creature_filter` — distinct from both `etb_filter`/`death_filter`/`combat_damage_filter`. Description: "Filter applied to the creature whose attack/death triggered this ability. Layer-resolved. Used by `AnyCreatureYouControlAttacks` and `AnyCreatureDies` (and any future trigger that fans out across attackers/dying-creatures)."

### Change 1: DSL — add field to two `TriggerCondition` variants

**File**: `crates/engine/src/cards/card_definition.rs`
**Action**:
```rust
// line ~2433
WheneverCreatureDies {
    controller: Option<TargetController>,
    #[serde(default)]
    exclude_self: bool,
    #[serde(default)]
    nontoken_only: bool,
    /// PB-N: optional filter on the dying creature's layer-resolved characteristics.
    /// Used for "Whenever a Vampire you control dies" (Crossway Troublemakers),
    /// "another black creature you control dies" (Teysa), etc. CR 603.10a.
    #[serde(default)]
    filter: Option<TargetFilter>,
},

// line ~2556
WheneverCreatureYouControlAttacks {
    /// PB-N: optional filter on the attacking creature's layer-resolved characteristics.
    /// Used for "Whenever a Dragon you control attacks" (Kolaghan, Dromoka), etc.
    /// CR 508.1m / CR 603.2.
    #[serde(default)]
    filter: Option<TargetFilter>,
},
```
Both fields are `#[serde(default)]` to avoid breaking the on-disk script schema. Existing card defs that use the unit/struct form without `filter` continue to compile because Rust requires explicit field syntax — see card-def fix list below for the 30+ files that need a 1-line edit.

### Change 2: Runtime — add field to `TriggeredAbilityDef`

**File**: `crates/engine/src/state/game_object.rs` (~l. 568)
**Action**:
```rust
pub struct TriggeredAbilityDef {
    pub trigger_on: TriggerEvent,
    pub intervening_if: Option<InterveningIf>,
    pub description: String,
    #[serde(default)]
    pub effect: Option<crate::cards::card_definition::Effect>,
    #[serde(default)]
    pub etb_filter: Option<ETBTriggerFilter>,
    #[serde(default)]
    pub death_filter: Option<DeathTriggerFilter>,
    #[serde(default)]
    pub combat_damage_filter: Option<crate::cards::card_definition::TargetFilter>,
    /// PB-N: layer-resolved filter applied to the triggering creature for
    /// `AnyCreatureYouControlAttacks` and `AnyCreatureDies`. Distinct from
    /// `combat_damage_filter` (which targets the damage-dealing creature on
    /// `AnyCreatureYouControlDealsCombatDamageToPlayer`). CR 508.1m / 603.10a.
    #[serde(default)]
    pub triggering_creature_filter: Option<crate::cards::card_definition::TargetFilter>,
    #[serde(default)]
    pub targets: Vec<crate::cards::card_definition::TargetRequirement>,
}
```

### Change 3: Enrichment — translate the new field

**File**: search for the function that maps `TriggerCondition` → `TriggeredAbilityDef`. Likely `crates/engine/src/cards/abilities_enrich.rs`. Find by:
```
Grep pattern="TriggerCondition::WheneverCreatureYouControlAttacks" -A=5 in cards/
```
**Action**: in the two match arms for `WheneverCreatureDies` and `WheneverCreatureYouControlAttacks`, copy the new `filter` into the resulting `TriggeredAbilityDef.triggering_creature_filter`. Two ~2-line edits.

### Change 4: Death-side dispatch

**File**: `crates/engine/src/rules/abilities.rs`
**Location**: after line 4174 (inside the `death_filter` block, before the intervening-if check at l. 4176)
**Action**:
```rust
// PB-N: triggering_creature_filter — subtype/color/keyword/type filter on the
// dying creature, evaluated against PRE-DEATH characteristics preserved on
// the graveyard object by move_object_to_zone (CR 603.10a LKI).
if let Some(ref filter) = trigger_def.triggering_creature_filter {
    let dying_obj = match state.objects.get(&dying_obj_id) {
        Some(o) => o,
        None => continue,
    };
    let dying_chars = crate::rules::layers::calculate_characteristics(state, dying_obj_id)
        .unwrap_or_else(|| dying_obj.characteristics.clone());
    // is_token guard — runtime field, not in Characteristics.
    if filter.is_token && !dying_is_token {
        continue;
    }
    if !crate::effects::matches_filter(&dying_chars, filter) {
        continue;
    }
}
```
Place this **after** the four existing `death_filter` checks and **before** the intervening-if. Order matters: cheap checks first (controller/exclude/nontoken), then the heavier `calculate_characteristics` call.

### Change 5: Attack-side dispatch

**File**: `crates/engine/src/rules/abilities.rs`
**Location**: line 5821-5836 — add a sibling check that runs `triggering_creature_filter` for `AnyCreatureYouControlAttacks` (the `combat_damage_filter` block already runs for both events because the outer `matches!` at 5809 covers both — but it MUST stay scoped to combat damage). The cleanest delta:

```rust
// inside the existing matches! block at line 5809:
if matches!(
    event_type,
    TriggerEvent::AnyCreatureYouControlAttacks
        | TriggerEvent::AnyCreatureYouControlDealsCombatDamageToPlayer
) {
    if let Some(attacking_id) = entering_object {
        if let Some(attacking_obj) = state.objects.get(&attacking_id) {
            if attacking_obj.controller != obj.controller {
                continue;
            }
            // Existing combat_damage_filter block — UNCHANGED, scoped to damage event.
            if event_type == TriggerEvent::AnyCreatureYouControlDealsCombatDamageToPlayer {
                if let Some(ref filter) = trigger_def.combat_damage_filter {
                    /* existing matches_filter call */
                }
            }
            // PB-N: triggering_creature_filter — applies to BOTH events (attack and damage),
            // but at the trigger-def author's discretion. Subtype/color/etc.
            if let Some(ref filter) = trigger_def.triggering_creature_filter {
                let attacking_chars = crate::rules::layers::calculate_characteristics(
                    state,
                    attacking_id,
                )
                .unwrap_or_else(|| attacking_obj.characteristics.clone());
                if filter.is_token && !attacking_obj.is_token {
                    continue;
                }
                if !crate::effects::matches_filter(&attacking_chars, filter) {
                    continue;
                }
            }
        } else {
            continue;
        }
    } else {
        continue;
    }
}
```

**Note for the runner**: the existing `combat_damage_filter` was previously checked unconditionally (for both events), which is *technically a latent bug* — the field name says "combat damage" but it ran on attacks too. PB-N tightens this by gating it on the damage-event arm. **Stop-and-flag if any existing card def relies on `combat_damage_filter` firing on attacks** (none do per the grep results — all 4 users are on damage triggers).

### Change 6: Hash dispatch

**File**: `crates/engine/src/state/hash.rs`
**Action 1** (~l. 2246): in `HashInto for TriggeredAbilityDef`, add:
```rust
self.triggering_creature_filter.hash_into(hasher);
```
right after the `combat_damage_filter` line.

**Action 2**: in `HashInto for TriggerCondition` (find by grepping the file for `WheneverCreatureDies`), add `filter.hash_into(hasher)` to both the `WheneverCreatureDies` arm and the `WheneverCreatureYouControlAttacks` arm. The latter changes from a unit variant to a struct variant — match arm shape changes from `=>` to `{ filter } =>`.

**Action 3**: bump hash sentinel from 3 → 4 in the schema-version constant (find by grepping for the existing `3u8.hash_into` or similar near the top of the impl). **Stop-and-flag if the bump policy is ambiguous** (we are at sentinel 3 post-PB-Q; the wip brief flagged this as unclear).

### Change 7: Card-def shape migration

The shape change `WheneverCreatureYouControlAttacks` (unit) → `WheneverCreatureYouControlAttacks { filter: Option<TargetFilter> }` is a **breaking source change** for every existing card def that uses the unit form. There are ~15-20 such files. Each needs a 1-line edit:

```rust
// before:
trigger_condition: TriggerCondition::WheneverCreatureYouControlAttacks,
// after:
trigger_condition: TriggerCondition::WheneverCreatureYouControlAttacks { filter: None },
```

Same for `WheneverCreatureDies` (struct variant existing — the `filter` field is `#[serde(default)]` so existing struct-literal users compile because Rust does NOT require all fields when they have defaults — **WAIT**: Rust does require all fields in a struct literal. `#[serde(default)]` only affects deserialization. So every existing `WheneverCreatureDies { controller, exclude_self, nontoken_only }` needs `filter: None,` added.) The runner MUST grep ALL existing users of both variants and add the field. Use:
```
Grep pattern="WheneverCreatureYouControlAttacks|WheneverCreatureDies " path="crates/engine/src/cards/defs/" output_mode="files_with_matches"
```
Expect 50+ files. This is mechanical but **must be exhaustive** — `cargo check` will complain at every site.

---

## Card Definition Fixes (in scope — 11 cards)

### Attack-side (4 cards)

#### `kolaghan_the_storms_fury.rs`
**Oracle text**: "Flying. Whenever a Dragon you control attacks, creatures you control get +1/+0 until end of turn. Dash {3}{B}{R}."
**Current state**: Already uses `WheneverCreatureYouControlAttacks` (line 26) with explicit TODO at line 24: "Dragon subtype filter not yet in DSL — over-triggers on non-Dragon attackers." Produces wrong game state.
**Fix**: change to `WheneverCreatureYouControlAttacks { filter: Some(TargetFilter { has_subtype: Some(SubType("Dragon".into())), ..Default::default() }) }`. Remove the TODO comment. Replace with PB-N citation.

#### `dromoka_the_eternal.rs`
**Oracle text**: "Flying. Whenever a Dragon you control attacks, bolster 2."
**Current state**: verify file exists; if not, this is a NEW card def to author.
**Fix**: same shape as Kolaghan, plus a `Bolster` effect (verify Bolster primitive exists; if not, demote to deferred).

#### `sanctum_seeker.rs`
**Oracle text**: "Whenever a Vampire you control attacks, each opponent loses 1 life and you gain 1 life."
**Current state**: verify file exists; if not, this is a NEW card def to author. Per the dsl-gap-audit memory, this card may already be partially authored with a TODO.
**Fix**: `filter: Some(TargetFilter { has_subtype: Some(SubType("Vampire".into())), ..default })` + LoseLife/GainLife effect sequence (already-shipped primitives).

#### `shared_animosity.rs`
**Oracle text**: "Whenever a creature you control attacks, it gets +1/+0 until end of turn for each other attacking creature that shares a creature type with it."
**Current state**: line 14-16 has TODO on the *trigger primitive itself* AND on the count-of-sharing-attackers effect.
**Fix**: PB-N closes the **trigger** half (no filter needed — `filter: None` because *any* creature triggers it). The buff-count-by-sharing-types **effect** half is a separate gap (`EffectAmount::CountAttackersSharingType` or similar). **Defer Shared Animosity to a later PB**: PB-N alone won't ship correct game state. Move to deferred list.

**Revised attack-side in-scope**: 3 cards (Kolaghan, Dromoka, Sanctum Seeker) + maybe a 4th if Dragon Tempest / Patron of the Vein-style attack triggers exist in defs already. Spot-check during impl.

### Death-side (7 cards)

#### `crossway_troublemakers.rs`
**Oracle text**: "Attacking Vampires you control have deathtouch and lifelink. Whenever a Vampire you control dies, you may pay 2 life. If you do, draw a card."
**Current state**: file exists, has TODO at line 41 ("optional 'may pay 2 life' cost not expressible") AND line 42 ("WheneverCreatureDies lacks subtype filter — fires on all your creatures").
**Fix**: PB-N closes line 42. Line 41 (optional payment cost) is a SEPARATE gap. **Partial fix**: add the subtype filter; the optional-payment TODO remains. The card moves from "fires on every creature death (broken)" to "fires on every Vampire death and unconditionally draws a card (still mildly wrong but closer)". **Net result still wrong game state** — must defer fully until optional payment ships. Move to deferred.

#### `teysa_orzhov_scion.rs`
**Oracle text**: "Whenever another black creature you control dies, create a 1/1 white Spirit creature token with flying."
**Current state**: abilities vec is empty (line 23: `abilities: vec![]`). TODO at lines 8-11: needs color filter on death trigger.
**Fix**: author the death trigger using `WheneverCreatureDies { controller: Some(TargetController::You), exclude_self: true, nontoken_only: false, filter: Some(TargetFilter { colors: Some([Color::Black].into()), ..default }) }` + `Effect::CreateToken(spirit_flying_token_spec())`. The sacrifice activated ability (the OTHER ability) remains TODO — but the card ships with a partial-correctness gain because the death trigger is its primary engine effect. **Decision**: ship as a *partial* card def — death trigger correct, sac ability omitted with explicit TODO retained. This matches PB policy of "no wrong game state" because the omitted ability simply doesn't fire (silent absence != wrong fire). Stop-and-flag if oversight prefers strict full-or-defer.

#### `serpents_soul_jar.rs`
**Oracle text**: "Whenever an Elf you control dies, exile it. {T}, Pay 2 life: Until end of turn, you may cast a creature spell from among cards exiled with this artifact."
**Current state**: verify file exists.
**Fix**: death trigger with `has_subtype: Some(Elf)` + `Effect::ExileTriggeringObject` (verify primitive exists; this references the *dying creature* by ObjectId — if no `ExileTriggeringObject` effect variant exists, this is a NEW gap and the card defers). The activated ability (cast-from-exile) is large and out of scope. **Defer if effect primitive missing**.

#### `pashalik_mons.rs`, `miara_thorn_of_the_glade.rs`, `omnath_locus_of_rage.rs`
Already listed in deferred (self-OR-filtered). **Out of scope.**

**Realistic death-side ship**: 1-2 cards (Teysa partial; Serpent's Soul-Jar conditional on exile primitive). The brief overcounted heavily.

### Holistic re-yield

After full oracle-text walk, the **truly clean PB-N ships**:

| # | Card | Side | Filter | Notes |
|---|------|------|--------|-------|
| 1 | Kolaghan, the Storm's Fury | Attack | Dragon | TODO at l.24 closed |
| 2 | Dromoka, the Eternal | Attack | Dragon | needs Bolster verification |
| 3 | Sanctum Seeker | Attack | Vampire | needs file verification |
| 4 | Teysa, Orzhov Scion | Death | black | partial — sac ability still TODO |

**Confirmed PB-N yield: 4 cards (1 already-broken fix + 3 new/verifying)**, NOT 11 as the optimistic count above. The honest number after walking oracle text is much smaller than the brief's 33 estimate (12% yield, well past the 60% discount).

**Recommendation to oversight**: PB-N is cheap to implement (~150 LOC engine + ~30 LOC card defs + 8 tests) and unblocks the *primitive* even if the immediate card yield is only 4. The primitive enables every future "creature you control of subtype X attacks/dies" card without further engine work. Worth shipping at this yield because the surface area is tiny. **Stop-and-flag if oversight wants to bundle PB-D (DamagedPlayer) into the same PB to amortize the test/review cost** — DamagedPlayer is a different dispatch site (player target filter, not creature subtype filter) so it doesn't unify, but the review cost is small enough that bundling is plausible.

### Deferred cards (final list — 11)

1. Pashalik Mons — self-OR-filtered death (compound).
2. Miara, Thorn of the Glade — self-OR-filtered death (compound).
3. Omnath, Locus of Rage — self-OR-filtered death + Landfall (PB-L).
4. Najeela, the Blade-Blossom — no controller restriction on attacker (different dispatch).
5. Athreos, God of Passage — owner-not-controller filter.
6. Skullclamp — equipped-creature LKI on death.
7. Crossway Troublemakers — needs optional payment cost primitive.
8. Shared Animosity — needs `EffectAmount::CountAttackersSharingType`.
9. Hellrider — already authorable post-PB-23 (no subtype filter); spot-check whether it's fully shipping. Not blocked by PB-N.
10. Battle Cry Goblin — Pack Tactics intervening-if (out of scope).
11. Serpent's Soul-Jar — pending `Effect::ExileTriggeringObject` verification. **Promote to in-scope only if the primitive exists.**

---

## Test Plan

All tests are in `crates/engine/tests/triggered_abilities_filter.rs` (new file) or appended to an existing trigger test file. Pattern: follow the existing combat-damage-filter tests for `WheneverCreatureYouControlDealsCombatDamageToPlayer { filter }`.

### MANDATORY (8)

1. **`test_pbn_attack_filter_subtype_match_fires`** — MANDATORY. CR 508.1m. Attacker is a Dragon, trigger source has `filter.has_subtype = Some(Dragon)`. Trigger fires. Asserts `state.pending_triggers.len() == 1`.
2. **`test_pbn_attack_filter_subtype_mismatch_no_fire`** — MANDATORY. CR 508.1m. Attacker is a Goblin, trigger source has `filter.has_subtype = Some(Dragon)`. Trigger does NOT fire.
3. **`test_pbn_attack_filter_color_match_fires`** — MANDATORY. CR 508.1m. Attacker is a black creature, filter has `colors = Some({Black})`. Fires. Confirms the filter primitive carries beyond subtypes.
4. **`test_pbn_death_filter_subtype_match_fires`** — MANDATORY. CR 603.10a. Vampire creature dies, trigger source has `filter.has_subtype = Some(Vampire)`. Fires. **Critically asserts pre-death LKI** by changing the dying creature's subtypes via continuous effect that ENDS at death — see the death LKI test below.
5. **`test_pbn_death_filter_subtype_mismatch_no_fire`** — MANDATORY. CR 603.10a. Goblin dies, filter wants Vampire. Does NOT fire.
6. **`test_pbn_death_filter_pre_death_lki_color`** — MANDATORY. CR 603.10a. Setup: a black creature is on the battlefield with a continuous "becomes white" effect. Effect ends as the creature dies (e.g., Path to Exile-ish). Filter requires `colors = Some({Black})`. Asserts the trigger DOES fire (because pre-death LKI sees black). This is the load-bearing LKI test that PB-Q4 retro warned us not to silently skip.
7. **`test_pbn_hash_parity_attack_filter`** — MANDATORY. Two `TriggeredAbilityDef` instances differing only in `triggering_creature_filter` hash to different values. Closes the PB-Q H1 retro lesson (every new dispatch field needs hash coverage).
8. **`test_pbn_kolaghan_end_to_end`** — MANDATORY. Real card. Cast Kolaghan, cast a Dragon, attack with the Dragon, assert the +1/+0 buff applies. Cast a non-Dragon, attack with it, assert NO buff. End-to-end through the harness, exercises full enrichment + dispatch + effect chain. **No silent skip — if Bolster blocks Dromoka, fall back to a card that uses Pump-style effects.**

### OPTIONAL (2)

9. **`test_pbn_death_filter_with_controller_you_combined`** — OPTIONAL. Both `death_filter.controller_you = true` AND `triggering_creature_filter.has_subtype = Some(Vampire)` set. Asserts AND-semantics: both must pass. Not strictly mandatory because each filter is tested in isolation, but cheap.
10. **`test_pbn_attack_filter_layer_resolved_subtype`** — OPTIONAL. A creature gains `Dragon` subtype via Arcane Adaptation-style continuous effect. Filter requires Dragon. Asserts that the filter sees the layer-resolved subtype (not the printed type). Closes the PB-S/PB-X "layer-resolved chars at every dispatch site" lesson.

**No silent skips**. Per PB-Q4 retro: every test labeled mandatory ships or PB-N does not close. If a test cannot be written because a primitive is missing (e.g., Arcane Adaptation in test 10), the test is downgraded to OPTIONAL up front and explicitly noted — never silently dropped during impl.

---

## Verification Checklist (runner — copy verbatim)

- [ ] `cargo check -p engine` clean after Change 1+2 (DSL + runtime field additions)
- [ ] `cargo check -p engine` clean after Change 7 (~50 mechanical card-def fixes — exhaustive)
- [ ] `cargo build --workspace` clean (catches replay-viewer + TUI exhaustive-match holes)
- [ ] All 8 mandatory tests pass
- [ ] At least 1 of 2 optional tests passes; the other is explicitly justified-skipped in the review file
- [ ] Hash sentinel bumped 3 → 4 (or stop-and-flag if policy is unclear)
- [ ] Hash parity test 7 passes
- [ ] No remaining TODO comments in the 4 in-scope card defs that mention "subtype filter" or "color filter on death/attack trigger"
- [ ] `cargo clippy --workspace -- -D warnings`
- [ ] `cargo fmt --check`
- [ ] Clean test count delta recorded in commit message (expect +8 to +10)

---

## Risks & Edge Cases

1. **Hash sentinel bump policy is ambiguous post-PB-Q.** PB-N adds a tagged field to two existing variants. The runner should stop-and-flag if there is no documented policy on whether tagged-field-additions require a sentinel bump. Conservative answer: bump to 4. If oversight says "no bump for serde-default field additions", revert.
2. **`combat_damage_filter` previously ran for attack events too.** PB-N tightens it to only run on the damage event. This is a latent semantic fix. Stop-and-flag if any of the 4 existing users (basri_ket, ajani_sleeper_agent, tyvar_kell, metastatic_evangel) actually relied on the old broader behavior.
3. **Death-side LKI assumes `move_object_to_zone` preserves Characteristics.** Verified by code reading but not by test. Test 6 is the load-bearing assertion. If it fails, PB-N collapses and the death-side dispatch needs an LKI snapshot capture point added — much larger PB.
4. **`Effect::ExileTriggeringObject` may not exist.** Serpent's Soul-Jar deferral hangs on this. The runner should grep at impl time and demote the card if missing, not silently substitute another effect.
5. **Card-def migration is mechanical but exhaustive.** The shape change on `WheneverCreatureYouControlAttacks` (unit → struct) breaks every existing user. Compile errors will be loud. Allow ~30 minutes of mechanical fixup time.
6. **Najeela / Athreos / Skullclamp / Pashalik / Omnath / Miara are explicitly out of scope.** The brief listed them all as candidates. Each represents a *different* dispatch shape (no-controller, owner, equipment-LKI, self-OR). Rolling them in doubles the PB. Stop-and-flag if oversight wants to bundle.
7. **Yield is small (4 cards).** PB-N is justified by future-proofing the primitive, not by immediate card output. Oversight may want to defer in favor of a higher-yield batch — flag this in the handoff summary.

---

## PB-L preamble note (carry-forward from Step 1)

PB-L (Landfall, rank 4 in the new slate) is **a real PB, not a stale-TODO sweep**. `ETBTriggerFilter` (`crates/engine/src/state/game_object.rs:549`) has `creature_only` but no `card_type_filter` or land-specific filter. `TriggerEvent` enum has `SelfEntersBattlefield` and `AnyPermanentEntersBattlefield` but no `LandEntersBattlefield`. Card defs `khalni_heart_expedition.rs` and `druid_class.rs` explicitly TODO on `WheneverLandEntersBattlefield`. Cheapest implementation: extend `ETBTriggerFilter` with `card_type_filter: Option<CardType>` (1 new field, ~3 dispatch sites). Per-rank yield estimate (~7 cards) holds. Recorded for oversight; PB-N does not act on this.
