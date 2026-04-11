# Primitive Batch Plan: PB-Q — ChooseColor (as-ETB color choice)

> ## DO NOT IMPLEMENT THIS SESSION
> Plan-phase deliverable. Implementation runs after oversight resolves the open
> questions at the bottom. End plan phase by updating `memory/primitive-wip.md`
> with new open questions; do not touch engine source.

**Generated**: 2026-04-11
**Primitive**: "As ~ enters, choose a color" replacement effect (CR 614.12) +
downstream color-aware filter and mana-doubling dispatch keyed off
`GameObject.chosen_color`.
**CR Rules**: 614.12 (replacement modifies how a permanent enters), 614.12a
(choice made before permanent enters), 105.1 (the five colors), 106.6 / 106.6a
(mana production replacements), 106.12 (tap-for-mana triggers), 608.2h (numeric
values determined once at apply time — relevant only for downstream effects
that read the chosen color, not the choice itself), 400.7 (object identity →
`chosen_color` resets on zone changes).
**Cards in scope**: 7 verified (3 stubbed in defs/, 4+ to be authored fresh).
**Dependencies**: builds on existing `chosen_creature_type` plumbing,
`ReplacementModification::ChooseCreatureType` (`replacement.rs:1440`),
`ManaWouldBeProduced` / `MultiplyMana` (`mana.rs:262`),
`EffectFilter::CreaturesYouControlWithColor(Color)` (already exists,
`continuous_effect.rs:170`).
**Deferred items from prior PBs**: none claimed for closure. PB-S residual LOWs
L01..L06, PB-M deferred items, Heritage Druid `TapNCreatures`, PB-Y Metallic
Mimic — all explicitly out of scope.

---

## Executive Summary

PB-Q ships **one** primitive: as-ETB "choose a color" as a replacement effect
modeled exactly after `ReplacementModification::ChooseCreatureType` (PB-X
template). It also adds the dispatch surface that the in-scope cards actually
read — a chosen-color analogue of `CreaturesYouControlOfChosenType` and a
chosen-color analogue of the existing `MultiplyMana` mana-doubling primitive.

| # | Component | Where | LOC | Required by |
|---|-----------|-------|-----|-------------|
| 1 | `GameObject.chosen_color: Option<Color>` | `state/game_object.rs` | ~5 | All in-scope cards |
| 2 | `ReplacementModification::ChooseColor(Color)` | `state/replacement_effect.rs` + dispatch in `rules/replacement.rs` | ~50 | All in-scope cards |
| 3 | `EffectFilter::CreaturesYouControlOfChosenColor` (dynamic, reads `source.chosen_color`) | `state/continuous_effect.rs` + `rules/layers.rs` | ~30 | Caged Sun, Gauntlet of Power, Painter's Servant (subset) |
| 4 | `ReplacementTrigger::ManaWouldBeProduced` color-restricted variant **OR** new `ReplacementModification::AddOneOfChosenColor` keyed off `source.chosen_color` | `state/replacement_effect.rs` + `rules/mana.rs` | ~50 | Caged Sun, Gauntlet of Power |
| 5 | Hash arms + ActivationCost / GameObject HashInto field audit | `state/hash.rs` | ~25 | All |
| 6 | `public_state_hash()` schema sentinel bump 2 → 3 | `state/hash.rs:6032` | 1 | All |

**Estimated total**: ~160 LOC engine + ~150 LOC card defs + ~14 unit tests
(including 2 mandatory full-dispatch tests for the new layer-aware filter and
the new mana-doubling dispatch).

---

## CR Rule Text (verified via MCP `mcp__mtg-rules__get_rule`)

### CR 614.12 — Replacements that modify how a permanent enters

> Some replacement effects modify how a permanent enters the battlefield. (See
> rules 614.1c–d.) Such effects may come from the permanent itself if they
> affect only that permanent... To determine which replacement effects apply
> and how they apply, check the characteristics of the permanent as it would
> exist on the battlefield, taking into account replacement effects that have
> already modified how it enters the battlefield.

**614.12a**: "If a replacement effect that modifies how a permanent enters the
battlefield requires a choice, that choice is made **before** the permanent
enters the battlefield."

**Engine implication**: The choice is committed before the GameObject's zone
becomes Battlefield. The same site as `ChooseCreatureType` in
`replacement.rs:1440` — we mutate the post-move object's `chosen_color` field,
which is the post-replacement battlefield state. Other ETB triggers / static
abilities that read `chosen_color` see the value already set. This is exactly
the bug PB-X C1 had with Obelisk authored as a triggered ability — DO NOT
repeat. Replacement only.

### CR 105.1 — The five colors

> The colors are white, blue, black, red, and green. White is represented by
> {W}, blue by {U}, black by {B}, red by {R}, and green by {G}.

Five values; map directly to existing `state::types::Color` enum (already
exists with `White, Blue, Black, Red, Green` — verified). No new color enum.

### CR 106.6a — Replacement effects increasing mana production

> Some replacement effects increase the amount of mana produced by a spell or
> ability. In these cases, any restrictions or additional effects created by
> the spell or ability will apply to all mana produced.

**Engine implication**: "Whenever a basic land is tapped for mana of the
chosen color, its controller adds an additional one mana of that color"
(Gauntlet of Power) is a replacement effect on mana production keyed by both
the **mana color produced** and the **chosen color** of the source permanent.
This is a strict generalization of the existing `MultiplyMana` dispatch in
`mana.rs:259-284`, which currently keys only on controller. Two viable
designs (open question 4 below).

### CR 400.7 — Object identity on zone change

`chosen_color` must reset to `None` when the object changes zones, exactly
like `chosen_creature_type` (`game_object.rs:1088`). This is automatic via
`Default` because new GameObjects are constructed fresh on zone change.

---

## Card Enumeration & Classification

### Method

Grep across `crates/engine/src/cards/defs/` for `chosen color`, `choose a color`,
`ChooseColor`. Cross-reference oracle text via `mcp__mtg-rules__lookup_card`
for cards mentioned in the WIP (Caged Sun, Gauntlet of Power, etc.) that are
not yet stubbed. **Implementer must redo this grep at impl-phase start in case
new defs landed.**

### Classification table

| # | Card | Stub exists? | Choice timing | Downstream dispatch needed | Verdict |
|---|------|-------------|---------------|----------------------------|---------|
| 1 | **Caged Sun** | NO | ETB (CR 614.12) | (a) chosen-color creature pump (`CreaturesYouControlOfChosenColor` filter + `ModifyBoth(1)`); (b) mana-doubling on lands tapped for chosen color | **THIS PRIMITIVE** |
| 2 | **Gauntlet of Power** | NO | ETB | (a) chosen-color creature pump for **all controllers** (`AllCreaturesOfChosenColor`, not "you control"); (b) mana doubling on **basic lands** tapped for chosen color | **THIS PRIMITIVE** |
| 3 | **Painter's Servant** | NO | ETB | "All cards that aren't on the battlefield, spells, and permanents are the chosen color in addition to their other colors." Adds chosen color to every object's color in nearly every zone — **CDA-shaped color-add layer effect** | **THIS PRIMITIVE — but flag as Tier 1 (verify scope)**, see below |
| 4 | **Throne of Eldraine** | YES (`throne_of_eldraine.rs`, abilities empty) | ETB | (a) "tapped, add four mana of chosen color" — mana ability that reads `source.chosen_color`; (b) mana-spending restriction "spend only on monocolored spells of that color" | **THIS PRIMITIVE for the choice; the spending-restriction is OUT OF SCOPE** (different primitive — see PB-Q-adjacent below) |
| 5 | **Temple of the Dragon Queen** | YES (`temple_of_the_dragon_queen.rs`, choice TODO) | ETB | mana ability "{T}: add one of chosen color" — reads `source.chosen_color` | **THIS PRIMITIVE** |
| 6 | **Utopia Sprawl** | NO | ETB **of an Aura** (CR 614.12 still applies — replacement on the Aura itself) | replacement on attached Forest's mana production: "add an additional one mana of the chosen color" | **THIS PRIMITIVE** — flag as Tier 1 (verify; see Aura ETB hop) |
| 7 | **Skrelv, Defector Mite** | YES (activated TODO) | **NOT ETB** — choice on each activation of an activated ability | sets `chosen_color` per activation (transient, until end of turn), grants hexproof-from-color, can't-be-blocked-by-color | **PB-Q-ADJACENT — ACTIVATED-TIME CHOICE, NOT ETB**. See "stop-and-flag" below. |
| 8 | **Nykthos, Shrine to Nyx** | YES (activated TODO) | **NOT ETB** — choice per activation of `{2},{T}: ...` | reads `EffectAmount::DevotionTo(color)` from a chosen color | **PB-Q-ADJACENT — ACTIVATED-TIME CHOICE** |
| 9 | **Three Tree City** | YES (already authored, choice deferred) | NOT ETB — choice per activation of `{2},{T}: ...` | reads chosen color for `AddManaOfAnyColorAmount` | **PB-Q-ADJACENT** |
| 10 | **Cavern of Souls** | YES (`cavern_of_souls.rs`) | ETB | choose a **creature type** — NOT a color | **NOT THIS PRIMITIVE** (oversight WIP misclassified — verified via MCP) |
| 11 | **Gauntlet of Might** | not relevant | n/a | static "Red creatures get +1/+1" / "Mountain tapped → add R" — **no choice** | **NOT A CHOICE PRIMITIVE** |
| 12 | **Extraplanar Lens** | not relevant | n/a | Imprint, not chosen color | **NOT A CHOICE PRIMITIVE** |

### IN PB-Q SCOPE (ETB choice cards)

1. Caged Sun (NEW)
2. Gauntlet of Power (NEW)
3. Throne of Eldraine (existing stub) — **partial author**: ETB choice + tap-for-4-of-chosen-color mana ability YES; mana-spending restriction TODO'd as out of scope.
4. Temple of the Dragon Queen (existing stub) — partial complete: choice + chosen-color mana ability YES.
5. Utopia Sprawl (NEW) — Aura ETB choice. **Tier 1 (verify)**: confirm AbilityDefinition::Replacement attached via Aura works through the existing aura attachment ordering (`resolution.rs` registers Aura attachment before static effects per `gotchas-rules.md` Enchant note).
6. Painter's Servant (NEW) — **Tier 1 (verify)**: the "all cards in nearly every zone become chosen color" effect needs a color-add layer modification that targets nearly the entire object set. This may require a SECOND new layer modification variant — flag in open questions; if verified out of reach, defer to a follow-up.

### PB-Q-ADJACENT (NOT this primitive — different choice timing)

These cards have "choose a color" as part of an **activated ability**, not as
ETB. The choice is made when the ability is **activated**, not when the
permanent enters. The downstream dispatch surface is similar but the choice
plumbing is different — the ability captures a color-choice parameter and
binds it through the resolution context (`EffectContext.chosen_color`,
analogous to `ctx.chosen_creature_type` at `effects/mod.rs:122`).

7. Skrelv, Defector Mite — activated `{W/P},{T}: Choose a color, ...`
8. Nykthos, Shrine to Nyx — activated `{2},{T}: Choose a color, ...`
9. Three Tree City — activated `{2},{T}: Choose a color, ...`
10. Throne of Eldraine's `{3},{T}: Draw two cards. Spend only mana of chosen
    color to activate this ability` — the **ETB-time** chosen color is read
    here, but the activation cost-restriction "spend only mana of chosen
    color" is a mana-spending-restriction primitive, NOT this batch.

**Recommendation**: defer all activated-time choose-color to a follow-up
**PB-Q2** (or fold into a generalized "choose-on-activation" cost-framework
batch). This keeps PB-Q narrow and consistent with PB-X's "small focused PBs"
directive. Open question 1.

### NOT A CHOICE PRIMITIVE

- Cavern of Souls (chooses a creature type, not a color)
- Gauntlet of Might (static, no choice)
- Extraplanar Lens (Imprint)
- Prismatic Omen (static color-aware, no choice — not in defs/, listed by
  user as illustrative)
- High Tide (no choice; triggers on Island tap)
- Bloodmark Mentor (uses existing `CreaturesYouControlWithColor(Color)` —
  static color, not chosen)

### Choose-A-Basic-Land-Type — STOP-AND-FLAG

Verified via MCP that **Utopia Sprawl is "choose a color", not "choose a
basic land type"** — the WIP and coordinator notes had this wrong. There is
**no card** in the in-scope set that needs choose-a-basic-land-type. PB-Q
does NOT introduce that primitive. If a future card needs it (e.g., Wild
Growth variants, Realmwalker?), spawn a separate micro-PB.

### Tier 1 (verify) markers

- **Painter's Servant**: the "in nearly every zone" scope of its color-add
  is broader than any current `LayerModification`. May need
  `LayerModification::AddColorDynamic { color_source: ChosenColorRef }` plus
  a layer-5 (color-modifying) ColorAdd dispatch path. Verify at impl-phase
  start; if it requires a second LayerModification variant, defer Painter's
  Servant to a follow-up and ship PB-Q with the other 5 cards.
- **Utopia Sprawl**: confirm Aura ETB-replacement timing; specifically that
  the Aura's own ETB replacement fires before its enchant-target attachment
  is recorded — or after, if the chosen color must be visible to the
  attached Forest's mana ability. Walk `resolution.rs` aura attachment site
  during impl phase. Per `gotchas-rules.md`, "Aura attachment ... must
  happen BEFORE `register_static_continuous_effects`" — confirm the same
  ordering applies to ETB replacements.
- **Gauntlet of Power**: the pump filter is **all creatures** (any
  controller) of the chosen color, NOT only "you control". Need a separate
  `EffectFilter::AllCreaturesOfChosenColor` (no controller restriction).
  Otherwise this card is incorrect.

---

## Engine Design

### 1. `GameObject.chosen_color` field

**File**: `crates/engine/src/state/game_object.rs` (after `chosen_creature_type`
at line ~1088)

```rust
/// CR 614.12 / CR 105.1: chosen color set by an as-ETB replacement
/// (Caged Sun, Gauntlet of Power, Throne of Eldraine, Utopia Sprawl,
/// Painter's Servant). Reset to `None` on zone change (CR 400.7); fresh
/// GameObjects default to `None`.
///
/// Read by:
/// - `EffectFilter::CreaturesYouControlOfChosenColor` / `AllCreaturesOfChosenColor`
///   in `rules/layers.rs` (chosen-color creature pump dispatch)
/// - mana-production replacement dispatch in `rules/mana.rs` (chosen-color
///   mana doubling)
/// - any future `Effect::AddManaOfChosenColor` / `Effect::ChosenColorRef`
#[serde(default)]
pub chosen_color: Option<crate::state::types::Color>,
```

`Default::default()` propagation: confirmed all `GameObject` constructors use
`..Default::default()` or already populate `chosen_creature_type: None`. The
new field gets `None` by default. Audit all 6 sites that explicitly construct
`chosen_creature_type: None` (`mod.rs:154,185`, `effects/mod.rs:3163,4034,4199,
6253,6999`) — add `chosen_color: None,` next to each. **PB-S H1 lesson**:
field-count the `HashInto for GameObject` impl after adding the field.

### 2. `ReplacementModification::ChooseColor(Color)`

**File**: `crates/engine/src/state/replacement_effect.rs` (after
`ChooseCreatureType(SubType)` at line 112)

```rust
/// CR 614.12 / CR 105.1: "As this enters, choose a color." Sets
/// `chosen_color` on the entering permanent. Deterministic fallback (since
/// interactive choice is M10): pick the most common color among permanents
/// the controller controls, falling back to the provided default.
///
/// Mirrors `ChooseCreatureType` exactly (`replacement.rs:1440`). The default
/// argument is the design-time fallback when no permanents exist to scan.
ChooseColor(crate::state::types::Color),
```

### 3. Replacement dispatch in `rules/replacement.rs`

After the `ChooseCreatureType` arm at line 1440-1475, add a parallel
`ChooseColor` arm. **CR justification**: 614.12a — choice is committed before
the permanent enters; we mutate the post-move object's `chosen_color` field,
which is the post-replacement battlefield state.

```rust
Some(ReplacementModification::ChooseColor(default_color)) => {
    // CR 614.12a: choice committed before the permanent enters.
    // Deterministic fallback: count colors of permanents the controller
    // controls (read from layer-resolved characteristics, CR 613.1d), pick
    // the most common, fall back to default if none.
    let chosen = {
        let mut color_counts: std::collections::HashMap<
            crate::state::types::Color,
            usize,
        > = std::collections::HashMap::new();
        for obj in state.objects.values() {
            if obj.controller == controller
                && matches!(obj.zone, crate::state::zone::ZoneId::Battlefield)
            {
                let chars = crate::rules::layers::calculate_characteristics(state, obj.id)
                    .unwrap_or_else(|| obj.characteristics.clone());
                for c in &chars.colors {
                    *color_counts.entry(*c).or_insert(0usize) += 1;
                }
            }
        }
        color_counts
            .into_iter()
            .max_by_key(|(_, count)| *count)
            .map(|(c, _)| c)
            .unwrap_or(default_color)
    };
    if let Some(obj) = state.objects.get_mut(&new_id) {
        obj.chosen_color = Some(chosen);
    }
}
```

**VERIFY at impl time**: the `chars.colors` field name in
`Characteristics` — grep `pub colors:` and use whatever the actual field
name is. If the field is `color_set` or similar, adjust.

### 4. `EffectFilter::CreaturesYouControlOfChosenColor` + `AllCreaturesOfChosenColor`

**File**: `crates/engine/src/state/continuous_effect.rs` (after
`CreaturesYouControlWithColor(Color)` at line 170)

```rust
/// CR 614.12 / CR 105.1: Creatures you control whose color set contains
/// the source's `chosen_color`. Used by Caged Sun's "Creatures you control
/// of the chosen color get +1/+1".
///
/// Reads `source.chosen_color` dynamically at layer-application time
/// (parallel to `CreaturesYouControlOfChosenType`). The source MUST have
/// `chosen_color = Some(_)`; otherwise the filter matches nothing.
CreaturesYouControlOfChosenColor,
/// CR 614.12 / CR 105.1: All creatures (any controller) whose color set
/// contains the source's `chosen_color`. Used by Gauntlet of Power's
/// "Creatures of the chosen color get +1/+1" — note this is NOT
/// "you control".
AllCreaturesOfChosenColor,
```

**File**: `crates/engine/src/rules/layers.rs` (after
`CreaturesYouControlOfChosenType` arm at line 908)

Two new match arms in `is_effect_active`. Pattern matches the existing
`CreaturesYouControlOfChosenType` (lines 908-925) but reads
`source.chosen_color` and the **layer-resolved** `chars.colors` rather than
`chars.subtypes`. Use `calculate_characteristics` to honor color-changing
effects per Layer 5 ordering (CR 613.1e).

```rust
EffectFilter::CreaturesYouControlOfChosenColor => {
    if obj_zone != ZoneId::Battlefield || !chars.card_types.contains(&CardType::Creature) {
        return false;
    }
    if let Some(source_id) = effect.source {
        let source = state.objects.get(&source_id);
        let source_controller = source.map(|s| s.controller);
        let chosen_color = source.and_then(|s| s.chosen_color);
        let obj_controller = state.objects.get(&object_id).map(|o| o.controller);
        source_controller.is_some()
            && source_controller == obj_controller
            && chosen_color
                .map(|c| chars.colors.contains(&c))
                .unwrap_or(false)
    } else {
        false
    }
}
EffectFilter::AllCreaturesOfChosenColor => {
    if obj_zone != ZoneId::Battlefield || !chars.card_types.contains(&CardType::Creature) {
        return false;
    }
    if let Some(source_id) = effect.source {
        let source = state.objects.get(&source_id);
        let chosen_color = source.and_then(|s| s.chosen_color);
        chosen_color.map(|c| chars.colors.contains(&c)).unwrap_or(false)
    } else {
        false
    }
}
```

**Layer dispatch site is named**: `is_effect_active` in `rules/layers.rs` at
line ~541 (the giant `match` on `EffectFilter`). **CR justification**:
613.1f (P/T modifications applied in layer 7c after layer 5 color resolution).
The filter reads `chars.colors` which is already layer-resolved by the time
`is_effect_active` is called (the layer pass walks layers 1→7 and color
modifications happen in layer 5 before P/T pump in layer 7c).

### 5. Mana production dispatch — Caged Sun / Gauntlet of Power second clause

**Two design alternatives — open question 4 below**.

#### Alternative A (preferred): generalize `ManaWouldBeProduced` filter

Extend `ReplacementTrigger::ManaWouldBeProduced` from
`{ controller: PlayerId }` to:

```rust
ManaWouldBeProduced {
    controller: PlayerId,
    /// Optional: only fire if the produced mana includes this color (read
    /// from `source.chosen_color` if `Self`, otherwise the literal Color).
    color_filter: Option<ChosenColorRef>,
    /// Optional: only fire if the source permanent is a basic land
    /// (Gauntlet of Power) or a land (Caged Sun).
    source_filter: Option<ManaSourceFilter>,
    /// What to add: "an additional one mana of that color".
}
```

Plus a new `ReplacementModification::AddOneManaOfChosenColor` that:
- reads `source.chosen_color` from the **replacement source** (Caged Sun /
  Gauntlet of Power object), NOT the tapped land
- adds 1 of that color to the mana pool

Dispatch in `rules/mana.rs:apply_mana_production_replacements` (line 259) is
extended to (a) iterate matching effects, (b) check the optional filters,
(c) call the modification handler.

**Why this is cleaner**: PB-E's `MultiplyMana` is multiplicative; this is
**additive** and **conditional on color**. They don't compose. A new
modification keeps the dispatch isolated.

#### Alternative B: new `ReplacementTrigger::ManaTappedForChosenColor`

A separate trigger variant fired by the same dispatch site, with a
straightforward modification `AddManaToPool { color: ChosenColorRef, amount: 1 }`.
Less generic but minimal surface change.

**Plan default: Alternative A** (filter on existing trigger, new
modification). Open question 4.

**File**: `crates/engine/src/rules/mana.rs:259-284`. The
`apply_mana_production_replacements` function currently returns a single
`u32` multiplier. It needs to also produce a list of additional `(ManaColor,
u32)` to **append** to the post-multiply mana pool. Refactor signature:

```rust
fn apply_mana_production_replacements(
    state: &GameState,
    player: PlayerId,
    source_perm: ObjectId,         // the LAND being tapped
    base_mana: &[(ManaColor, u32)], // produced before replacement
) -> (u32, Vec<(ManaColor, u32)>) // (multiplier, additions)
```

Caller (around line 250) passes the base mana set after step 9 ("compute
mana"), applies the multiplier, then appends the additions. **CR
justification**: 106.6a — replacement effects increase the amount of mana
produced; restrictions/additional effects apply to all produced mana. The
multiplier acts on base; additions are separate per CR 106.6a (each mana
of the chosen color triggers one additional mana — but for Caged Sun
"Whenever a land's ability **causes you to add one or more mana of the
chosen color**, add an additional one mana", the addition is **once per
trigger event**, not per mana — open question 5).

### 6. Hash arms + version bump

**File**: `crates/engine/src/state/hash.rs`

| Item | Discriminant | Site | Notes |
|------|-------------|------|-------|
| `EffectFilter::CreaturesYouControlOfChosenColor` | 34 | line ~1272 (after PB-X ended at 33) | next free per PB-X plan: 34 |
| `EffectFilter::AllCreaturesOfChosenColor` | 35 | line ~1272 | |
| `ReplacementModification::ChooseColor(c)` | next free after `MultiplyMana` (line 1808) | hash the `Color` byte | count exactly at impl |
| `GameObject.chosen_color` | n/a (struct field) | `HashInto for GameObject` | **CRITICAL** — PB-S H1 / PB-X open Q5 lesson. Field-count the impl. |
| `ReplacementTrigger::ManaWouldBeProduced` field changes (Alt A) | extend existing arm | hash the new fields | |
| Schema sentinel | bump 2 → **3** | `state/hash.rs:6032` | per PB-X precedent |

**PB-S H1 lesson** must be reinforced: the implementer MUST count fields in
`HashInto for GameObject` against the struct definition after adding
`chosen_color`. PB-S forgot `once_per_turn`; PB-X's review-fix reminded us.

### 7. Exhaustive match audit

| Site | File | Action |
|------|------|--------|
| `is_effect_active` | `rules/layers.rs:541` | Add 2 EffectFilter arms |
| `HashInto for EffectFilter` | `state/hash.rs:1196` (around line 1272) | Add 2 hash arms |
| `HashInto for ReplacementModification` | `state/hash.rs:1808` | Add 1 arm for ChooseColor |
| `HashInto for ReplacementTrigger` | `state/hash.rs` (search `ManaWouldBeProduced`) | If Alt A, hash new fields |
| `apply_replacement_to_etb` | `rules/replacement.rs:1440` | Add ChooseColor arm |
| `apply_mana_production_replacements` | `rules/mana.rs:259` | Refactor signature; add additions list |
| Mana dispatch caller | `rules/mana.rs:~250` | Append additions to mana pool |
| `HashInto for GameObject` | `state/hash.rs` (search `chosen_creature_type`) | Add `chosen_color.hash_into(hasher)` |
| GameObject default sites | `effects/mod.rs:3163,4034,4199,6253,6999`; `state/mod.rs:154,185` | Add `chosen_color: None,` |
| TUI / replay-viewer | `tools/tui/src/play/panels/stack_view.rs`, `tools/replay-viewer/src/view_model.rs` | **VERIFY** none match on `EffectFilter` / `ReplacementModification` exhaustively. PB-X plan verified `EffectFilter` has no cross-crate matches. Re-grep at impl time. |

**Per `memory/MEMORY.md` Behavioral Gotchas**: replay-viewer has exhaustive
matches on `StackObjectKind` AND `KeywordAbility`. Neither enum is touched
by PB-Q. Confirm at impl-build time with `cargo build --workspace`.

---

## Card Definition Patches

### caged_sun.rs (NEW)

Two abilities: ETB choose-color replacement, then a static pump using
`CreaturesYouControlOfChosenColor` and a replacement on mana production
keyed off the chosen color.

```rust
// Caged Sun — {6} Artifact
// As this artifact enters, choose a color.
// Creatures you control of the chosen color get +1/+1.
// Whenever a land's ability causes you to add one or more mana of the
// chosen color, add an additional one mana of that color.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("caged-sun"),
        name: "Caged Sun".to_string(),
        mana_cost: Some(ManaCost { generic: 6, ..Default::default() }),
        types: types(&[CardType::Artifact]),
        oracle_text: "...".to_string(),
        abilities: vec![
            // CR 614.12: as-ETB replacement (NOT a triggered ability — PB-X C1 lesson)
            AbilityDefinition::Replacement {
                trigger: ReplacementTrigger::WouldEnterBattlefield {
                    filter: ObjectFilter::Any,
                },
                modification: ReplacementModification::ChooseColor(Color::White),
                is_self: true,
                unless_condition: None,
            },
            // Static: chosen-color creature pump
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::PtModify,
                    modification: LayerModification::ModifyBoth(1),
                    filter: EffectFilter::CreaturesYouControlOfChosenColor,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    condition: None,
                },
            },
            // Static: mana doubling on chosen-color land mana (PB-Q new dispatch)
            // shape per Alternative A — see open question 4
            AbilityDefinition::Replacement {
                trigger: ReplacementTrigger::ManaWouldBeProduced {
                    controller: PlayerId(0), // bound at registration
                    color_filter: Some(ChosenColorRef::SelfChosen),
                    source_filter: Some(ManaSourceFilter::AnyLand),
                },
                modification: ReplacementModification::AddOneManaOfChosenColor,
                is_self: false,
                unless_condition: None,
            },
        ],
        ..Default::default()
    }
}
```

### gauntlet_of_power.rs (NEW)

Same as Caged Sun but: pump filter is `AllCreaturesOfChosenColor` (any
controller); mana doubling source filter is `ManaSourceFilter::BasicLand`.

### throne_of_eldraine.rs (PATCH)

Replace empty abilities vec with:
1. ETB ChooseColor replacement.
2. Activated `{T}: Add four mana of the chosen color`. Reads
   `source.chosen_color` via a new `Effect::AddManaOfChosenColor` or via
   plumbing from `obj.chosen_color` into the mana ability — open question 6.
3. `{3},{T}: Draw two cards` activated ability — author with TODO marker
   for the mana-spending restriction (out of scope).

### temple_of_the_dragon_queen.rs (PATCH)

Add ETB ChooseColor replacement and `{T}: Add one of chosen color` mana
ability. The ETB-tapped-unless replacement already exists; add
`ChooseColor` as a second replacement (CR 616.1: multiple ETB replacements,
controller chooses order).

### utopia_sprawl.rs (NEW)

Aura. ETB ChooseColor replacement (on the Aura object). Mana doubling
replacement keyed off the **enchanted Forest** when tapped — needs a
`ManaSourceFilter::EnchantedPermanent` or to bind the source to
`obj.attached_to` at registration. **Tier 1 (verify)** — flag in open
question 7.

### painters_servant.rs (NEW or DEFERRED)

ETB ChooseColor + Layer 5 color-add. The color-add filter scope is
"all cards that aren't on the battlefield, spells, and permanents" — in
practice, every object. Needs a new `LayerModification::AddColorDynamic`
variant. **Tier 1 (verify)** — if the implementer determines this needs a
second LayerModification variant beyond PB-Q's scope, **defer Painter's
Servant to a follow-up PB**. Do NOT silently add a second
LayerModification variant inside PB-Q.

---

## Unit Tests

**File**: `crates/engine/tests/primitive_pb_q.rs` (new)
**Pattern**: follow `crates/engine/tests/primitive_pb_x.rs` and the
replacement-effect ETB tests in `tests/replacement_effects.rs`.

### Replacement tests (Component 2)

1. `test_choose_color_replacement_sets_field` — cast Caged Sun; assert the
   resulting battlefield object has `chosen_color = Some(_)` immediately
   after resolution, before any priority window. CR 614.12a.
2. `test_choose_color_deterministic_fallback_picks_majority` — controller
   has 2 white creatures + 1 blue creature; cast Caged Sun (default red);
   assert chosen_color = White.
3. `test_choose_color_default_when_no_permanents` — empty board; assert
   chosen_color = the printed default.
4. `test_choose_color_resets_on_zone_change` — Caged Sun on battlefield
   with chosen_color = White; bounce it to hand; recast; assert the new
   GameObject's `chosen_color` is fresh (not the prior White).
   CR 400.7 / 614.12.

### Filter dispatch tests (Component 4) — **MANDATORY full-dispatch**

5. **`test_caged_sun_full_dispatch_pumps_chosen_color_creatures`** —
   FULL-DISPATCH per `memory/conventions.md` standing rule.
   Setup: 1 white creature, 1 blue creature, 1 red creature (all
   controlled by P0). Cast Caged Sun via `Command::CastSpell`. Resolve
   ETB replacement. Assert: P0 white creature has +1/+1 via
   `calculate_characteristics`; blue and red unchanged. **No direct
   `is_effect_active` invocation; full Command path only.**
6. `test_gauntlet_of_power_pumps_all_controllers_chosen_color` —
   full-dispatch. P0 controls 1 red creature; P1 controls 1 red and 1
   green. Cast Gauntlet of Power (default red). Assert: both red
   creatures have +1/+1 (any controller); green unchanged.
   Discriminates `AllCreaturesOfChosenColor` from `CreaturesYouControlOfChosenColor`.
7. `test_chosen_color_filter_no_choice_matches_nothing` — manually
   construct a Static effect with `CreaturesYouControlOfChosenColor` but
   no `chosen_color` set on the source. Assert no creatures get pumped.
   Defends against the PB-X observability-window failure mode in the
   filter direction.

### Mana doubling dispatch tests (Component 5) — **MANDATORY full-dispatch**

8. **`test_caged_sun_doubles_chosen_color_land_mana`** — FULL-DISPATCH.
   P0 controls Caged Sun (chosen White) and a Plains. Tap Plains for
   mana. Assert: P0 mana pool has 2 W (1 from Plains + 1 from Caged Sun
   replacement). CR 106.6a.
9. `test_caged_sun_does_not_double_other_color_mana` — same setup;
   Caged Sun chose White; tap a Mountain for R. Assert: P0 mana pool
   has 1 R (no doubling). Discriminates the color filter.
10. `test_gauntlet_of_power_only_doubles_basic_lands` — P0 controls
    Gauntlet of Power (chosen Red), a Mountain (basic), and Sulfur
    Falls (nonbasic that taps for R). Tap each. Assert: Mountain
    produces 2 R (doubled); Sulfur Falls produces 1 R (not doubled).
    Discriminates `ManaSourceFilter::BasicLand` from `AnyLand`.
11. `test_caged_sun_chosen_color_change_via_bounce_recast` — Caged Sun
    on bf with chosen Red; bounce to hand; recast (deterministic
    fallback now picks differently because the board changed). Assert
    new chosen color is correct, mana doubling now triggers for the
    new color.

### Card-level integration tests

12. `test_throne_of_eldraine_etb_chosen_color_and_mana_ability` —
    cast Throne; choose color (deterministic); activate `{T}: add four`.
    Assert mana pool has 4 of the chosen color.
13. `test_temple_of_the_dragon_queen_etb_chosen_color` — Temple ETB
    via lay-land; verify `chosen_color` set; activate `{T}: add one`;
    verify pool.
14. `test_utopia_sprawl_aura_etb_chosen_color` — Tier 1 verification
    test; cast Utopia Sprawl on a Forest; verify `chosen_color` set on
    the Aura; tap Forest; verify additional mana of chosen color.

### Hash test

15. `test_chosen_color_hash_field_audit` — construct two GameObjects
    differing only in `chosen_color`; hash both via the public hash
    path; assert hashes differ. **Defends against PB-S H1 failure
    mode** (forgot `once_per_turn` in HashInto).

**Test count**: 15. Trim to ≥11 if duplicative; the two **bolded
full-dispatch tests** (5 and 8) are MANDATORY per
`memory/conventions.md` standing rule and may not be removed.

---

## Discriminant Chain

| Enum | New variants | Discriminants | Next free after PB-Q |
|------|--------------|---------------|----------------------|
| `EffectFilter` | `CreaturesYouControlOfChosenColor`, `AllCreaturesOfChosenColor` | 34, 35 | 36 |
| `ReplacementModification` | `ChooseColor(Color)`, `AddOneManaOfChosenColor` (Alt A) | TBD (count at impl) | TBD+2 |
| `ReplacementTrigger` (Alt A) | extends `ManaWouldBeProduced` with new fields | n/a (struct fields, not new variants) | n/a |
| `Color` field on `GameObject` | n/a — struct field | n/a | n/a |
| **Hash version sentinel** | bump 2 → **3** | `state/hash.rs:6032` | 4 |

---

## Risks & Edge Cases

1. **`chars.colors` field name** — verify at impl time. If the field is
   `color_set` or wrapped differently, adjust both filter arms and the
   replacement's deterministic fallback.
2. **Aura ETB replacement timing** (Utopia Sprawl) — confirm that the Aura
   object's own ETB replacement fires before its enchant-target attachment
   is recorded. Walk `resolution.rs` aura attachment site. Per
   `gotchas-rules.md` Enchant note, attachment happens before
   `register_static_continuous_effects` — ETB replacement should be even
   earlier. Verify.
3. **Painter's Servant scope** — color-add to "every card in nearly every
   zone" requires a new `LayerModification` (color-modify in Layer 5). If
   verifying in impl phase shows it crosses the PB-Q boundary, defer
   Painter's Servant. Do NOT silently expand scope.
4. **Multiple Caged Suns / Gauntlets of Power** (CR 106.6a additivity) —
   each replacement registers independently and fires once per matching
   tap. Two Caged Suns (both White) tapping a Plains: 1 W (base) + 1 W
   (CS#1) + 1 W (CS#2) = 3 W. Verify in test 11 or add test 16.
5. **Tap-for-multiple-colors lands** (City of Brass, Mana Confluence) —
   the produced mana is a single color per tap; the chosen color filter
   should match if the chosen color is a subset. Verify in test 9 or
   add a test variant.
6. **PB-S H1 hash regression** — adding `chosen_color` to GameObject
   without updating `HashInto for GameObject` causes silent hash
   collisions. Implementer must field-count.
7. **Painter's Servant + Caged Sun chosen-color creature pump** —
   Painter's Servant turns every creature into the chosen color; Caged
   Sun's filter then matches everything. This is the correct
   interaction per CR 105.1 + Painter's Servant text. Layer 5 (color)
   resolves before Layer 7c (P/T pump). Verify in a follow-up if
   Painter's Servant ships.
8. **Object identity on bounce** (CR 400.7) — `chosen_color` is on
   the GameObject and resets to None on zone change because new
   GameObjects are constructed fresh. Test 4 verifies.
9. **Multiple ETB replacements with choices** (CR 614.12b) — Temple
   of the Dragon Queen has two ETB replacements (etb-tapped-unless and
   choose-color). Controller picks order. Existing ETB replacement
   ordering machinery handles this; no new code.
10. **`MultiplyMana` × `AddOneManaOfChosenColor` interaction** — does
    Caged Sun's "+1 W" stack with Mana Reflection's "x2"? Per CR 616.1
    the controller chooses order. If Reflection applies first: 1 W → 2 W
    base, then Caged Sun adds 1 W = 3 W total. If Caged Sun first: 1 W +
    1 W = 2 W, then Reflection doubles → 4 W. The apply-once-then-recheck
    rule (CR 616.1) means only one ordering is correct. Verify against CR
    616.1 example at impl time. Open question 8.

---

## Stop-and-Flag Findings

- **F1 — Coordinator scope notes had two factual errors verified via MCP**:
  - **Cavern of Souls is choose-a-creature-type, not choose-a-color.** Verified
    via `mcp__mtg-rules__lookup_card`. WIP listed it as a ChooseColor exemplar;
    it is not.
  - **Utopia Sprawl is choose-a-color, not choose-a-basic-land-type.** Verified
    via `mcp__mtg-rules__lookup_card`. The land it enchants is restricted to
    Forest by Enchant Forest, but the choice it makes is a **color**, not a
    basic land type. Coordinator notes flagged this as "likely a different
    primitive — STOP AND FLAG before bundling"; flagging now: **it's actually
    THIS primitive**, no bundling required.

- **F2 — Activated-time choose-color is a different primitive**. Skrelv,
  Defector Mite, Nykthos Shrine to Nyx, Three Tree City (third ability), and
  Throne of Eldraine's draw-cards activation all need choose-color as part of
  an activated-ability cost or effect, not as ETB replacement. This is a
  **different choice timing** and should be a separate batch (PB-Q2). Plan
  defers them. Open question 1.

- **F3 — Painter's Servant may need a fourth-primitive surface (color-add
  layer modification)**. Tier 1 verify; defer if confirmed out of scope.

- **F4 — Gauntlet of Power requires a new "all controllers" filter variant**
  distinct from the existing "you control" pattern. This is the
  `AllCreaturesOfChosenColor` variant in the design. Caged Sun uses
  the "you control" variant. Both ship in PB-Q.

- **F5 — Mana doubling needs new dispatch surface, not reuse of MultiplyMana**.
  PB-E's `MultiplyMana` is multiplicative and unconditional on color.
  Caged Sun / Gauntlet are additive and color-conditional. They don't compose
  cleanly into the existing trigger; PB-Q adds new replacement-trigger
  filters and a new modification variant.

- **F6 — `Color` enum already exists** at `state/types.rs:7`. No new color
  enum needed. Use it directly.

---

## Open Questions for Oversight

1. **Defer activated-time choose-color (Skrelv, Nykthos, Three Tree City,
   Throne's draw activation) to PB-Q2?** Plan default: yes — different
   choice timing, different plumbing (`EffectContext.chosen_color` per
   activation, not GameObject field). Confirm or override.

2. **Painter's Servant disposition.** Plan default: Tier 1 verify; if the
   color-add layer modification proves to be a second new primitive, defer
   to a follow-up micro-PB. Confirm or override.

3. **Gauntlet of Might out of scope?** It's static "Red creatures" (no
   choice). Could be authored alongside Gauntlet of Power as a free win
   using the existing `EffectFilter::CreaturesYouControlWithColor(Color)`,
   but the `AllCreaturesWithColor` variant doesn't exist yet — would need
   a static "all creatures of a fixed color" variant. Plan default: defer
   (not a choice primitive). Override if free-win desired.

4. **Mana doubling design — Alt A (extend trigger filter) vs Alt B (new
   trigger variant)?** Plan default: Alt A. Both work; Alt A is more
   general. Confirm.

5. **Caged Sun "additional one mana" semantics**: the oracle text says
   "Whenever a land's ability causes you to add one or more mana of the
   chosen color, add **an additional one mana of that color**." Is this
   one extra mana **per trigger event** (per land tap) or **per mana
   produced** (e.g., a land that taps for 2 of the chosen color triggers
   2 extra)? Per Scryfall ruling history (verify with MCP at impl time),
   it's **per trigger event** — one extra per tap, regardless of how
   much the land produced. Confirm at impl phase via
   `mcp__mtg-rules__search_rulings("Caged Sun additional mana")`.

6. **Throne of Eldraine `{T}: Add four mana of the chosen color`**: needs
   either a new `Effect::AddManaOfChosenColor { amount: u32 }` variant or
   plumbing of `obj.chosen_color` into the existing `Effect::AddMana`
   resolution path. Plan default: new effect variant
   `Effect::AddManaOfChosenColor`, since it parallels `EffectAmount`'s
   chosen-type pattern. Confirm.

7. **Utopia Sprawl Aura ETB ordering**: confirm at impl time that the
   Aura's own ETB replacement fires before its enchant-target attachment
   ordering. If it must fire after attachment (so the chosen color is
   visible to mana abilities of the attached Forest), reorder.

8. **Caged Sun × Mana Reflection ordering** (CR 616.1): the controller
   chooses order. Plan adds test 11 to lock in one canonical order; the
   choice depends on which order is the deterministic fallback for now.
   Confirm fallback policy: "controller-of-mana-source picks; default to
   apply additive (Caged Sun) before multiplicative (Reflection)".

9. **Hash version sentinel bump 2 → 3**: confirm policy. PB-X bumped to 2;
   PB-Q proposes 3.

10. **`PB-Q2` naming**: if oversight approves the activated-time deferral
    (Q1), reserve the name PB-Q2 in `docs/primitive-card-plan.md` and
    update the dispatch table.

---

## Verification Checklist

- [ ] `GameObject.chosen_color` field added; all GameObject construction sites
      updated; `HashInto for GameObject` field-counted and updated
- [ ] `ReplacementModification::ChooseColor(Color)` variant added with hash arm
- [ ] `apply_replacement_to_etb` ChooseColor arm added in `replacement.rs`
- [ ] `EffectFilter::CreaturesYouControlOfChosenColor` and
      `AllCreaturesOfChosenColor` variants added with layer arms + hash arms
- [ ] Mana production replacement dispatch extended (Alt A or B per Q4)
- [ ] `apply_mana_production_replacements` signature refactored to return
      additions list
- [ ] All 6 in-scope card defs author cleanly (Caged Sun, Gauntlet of Power,
      Throne of Eldraine PATCH, Temple of the Dragon Queen PATCH, Utopia
      Sprawl, Painter's Servant if Q2 approved)
- [ ] Activated-time choose-color cards (Skrelv, Nykthos, Three Tree City)
      remain stubbed and documented as PB-Q2 deferrals
- [ ] 11+ unit tests in `tests/primitive_pb_q.rs` pass, including the **2
      mandatory full-dispatch tests** (`test_caged_sun_full_dispatch_pumps_chosen_color_creatures`
      and `test_caged_sun_doubles_chosen_color_land_mana`)
- [ ] `cargo test --all` green
- [ ] `cargo clippy -- -D warnings` clean
- [ ] `cargo build --workspace` builds replay-viewer + TUI (verify no
      cross-crate exhaustive-match cascade on the new variants)
- [ ] Hash version sentinel bumped 2 → 3 at `state/hash.rs:6032`
- [ ] No new TODOs introduced in PB-Q-scope card defs
- [ ] `memory/primitive-wip.md` advanced from `plan` → `implement`
- [ ] Carry-forward LOWs from PB-S NOT touched (out of scope)
- [ ] PB-X deferred items NOT touched (out of scope)

---

## Files Touched (impl-phase preview)

| File | Reason | Approx LOC |
|------|--------|------------|
| `crates/engine/src/state/game_object.rs` | Add `chosen_color: Option<Color>` field | +5 |
| `crates/engine/src/state/replacement_effect.rs` | Add `ChooseColor(Color)` + Alt A trigger filter fields + `AddOneManaOfChosenColor` | +30 |
| `crates/engine/src/state/continuous_effect.rs` | Add 2 EffectFilter variants | +20 |
| `crates/engine/src/rules/replacement.rs` | Add ChooseColor arm at line 1440 | +35 |
| `crates/engine/src/rules/layers.rs` | Add 2 filter match arms at line 925 | +30 |
| `crates/engine/src/rules/mana.rs` | Refactor `apply_mana_production_replacements`, add color-conditional dispatch | +50 |
| `crates/engine/src/state/hash.rs` | 4 hash arms (2 EffectFilter, 1 ReplMod, GameObject field) + version bump | +20 |
| `crates/engine/src/effects/mod.rs` (if Q6 yes) | Add `Effect::AddManaOfChosenColor` variant + dispatch | +30 |
| `crates/engine/src/cards/defs/caged_sun.rs` | NEW | ~50 |
| `crates/engine/src/cards/defs/gauntlet_of_power.rs` | NEW | ~50 |
| `crates/engine/src/cards/defs/utopia_sprawl.rs` | NEW | ~40 |
| `crates/engine/src/cards/defs/throne_of_eldraine.rs` | PATCH | ~30 |
| `crates/engine/src/cards/defs/temple_of_the_dragon_queen.rs` | PATCH | ~15 |
| `crates/engine/src/cards/defs/painters_servant.rs` (if Q2 approved) | NEW or DEFER | ~40 |
| `crates/engine/tests/primitive_pb_q.rs` | NEW test file | ~450 |
| **TOTAL** | | **~900 LOC** |

Engine code (excluding card defs + tests): **~220 LOC**, slightly above the
PB-X "~140 LOC engine" baseline because PB-Q ships dispatch surface for two
distinct read sites (filter + mana production), not just substitution.

---

## DO NOT IMPLEMENT THIS SESSION

Plan ends here. Implementation runs in a separate session after oversight
resolves the 10 open questions above. The implementer's first acts at
impl-phase start MUST be:

1. Re-grep `defs/` for `chosen color` / `choose a color` / `ChooseColor` to
   detect any new card stubs added between plan and impl phases.
2. Re-verify the `chars.colors` field name in `Characteristics`.
3. Re-verify cross-crate exhaustive-match audit for the touched enums.
4. Re-verify the existing `ReplacementModification::ChooseCreatureType`
   dispatch site at `replacement.rs:1440` is unchanged (the PB-Q ChooseColor
   arm is a parallel of it).
5. Field-count `HashInto for GameObject` against `GameObject` definition
   AFTER adding `chosen_color` (PB-S H1 lesson).
6. Re-verify CR 106.6a / 614.12 / 614.12a via `mcp__mtg-rules__get_rule`.
