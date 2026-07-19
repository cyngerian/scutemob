# Primitive Batch Plan: PB-OS5 (OOS-EF4-1) — Dynamic relative-count `EffectAmount`

**Generated**: 2026-07-19
**Primitive**: A single new `EffectAmount` variant — `OtherAttackersSharingCreatureType { relative_to: EffectTarget }` — that at RESOLUTION counts attacking creatures whose **layer-resolved** creature-type set intersects the *relative* (triggering) creature's layer-resolved creature-type set, excluding the relative creature. Resolves via the spell-effect path (`resolve_amount`), fed into a UEOT pump through the existing `LayerModification::ModifyPowerDynamic` substitution.
**CR Rules**: 508.1/508.1m (declare attackers / attack triggers), 509 (attackers set), 205.3m (creature types; Changeling shares all types), 613.1d/613.3 (layer-resolved characteristics; CDAs), 107.3f ("for each" magnitude chosen/defined at resolution), 109.1/603.2 ("other"/exclude-self), 611.2a (continuous-effect subject).
**Cards affected**: 4 (1 existing→Complete, 1 new→Complete, 1 existing partial-improvement, 1 new→partial)
**Dependencies**: PB-EF3 (`EffectContext.triggering_creature_id` threading), PB-EF4 (`EffectFilter::TriggeringCreature`), PB-N (`WheneverCreatureYouControlAttacks`), PB-AC3 (`AttackingCreatureCount`, `ModifyPowerDynamic` substitution) — all SHIPPED.
**Deferred items from prior PBs**: OOS-EF4-1 (this batch's driver). No other carry-forward implicates this primitive.

**TODO sweep result**: Ran `Grep "OOS-EF4|shares a creature type|for each other attacking|attacking Goblin|other attacking|CountOtherAttackers|CountMatchingRelativeTo"` over `crates/card-defs/src/defs/`. **1 forced add found**: `goblin_rabblemaster.rs` (line 54 TODO `"+1/+0 per attacking Goblin — count-based pump not in DSL"`, plus a completeness note that pre-diagnoses the exact fix). Added to roster below (not in original PB brief). `hero_of_bladehold.rs` / `signal_pest.rs` / `goblin_wardriver.rs` matched on **Battle cry** ("each other attacking creature gets +1/+0") which is a *keyword* handled elsewhere, NOT this count primitive — excluded. `path_of_ancestry.rs` matched "shares a creature type" but it is a mana-spend spell-cast condition, not an attacker count — excluded.

---

## Design decision (summary for the coordinator)

**Only ONE candidate needs new engine surface.** The research surfaced that `AttackingCreatureCount { controller, filter }` (PB-AC3, discriminant 19) already exists and already honors `filter.exclude_self` (keyed to `ctx.source`), layer-resolved subtypes, phased-out exclusion, and `PlayerTarget::EachPlayer` ("all attackers") scope. That covers piledriver, rabblemaster, and (via `PermanentCount`) muxus with **zero** new variants.

The genuine gap is **shared_animosity** alone, because its trigger source (the enchantment) is NOT the subject (the attacking creature), so:
- the count must read the **triggering creature's** layer-resolved subtypes (not `ctx.source`'s), and
- "other" must exclude the **triggering creature** (not `ctx.source`).

Neither is expressible with `AttackingCreatureCount`'s `ctx.source`-keyed `exclude_self`, and "shares ≥1 creature type with object X" is inherently a **relative** predicate that cannot live as a static `TargetFilter` field without spraying runtime-relationship checks across dozens of match/validate sites. So it gets one purpose-built `EffectAmount` variant.

**Chosen variant shape** (minimal, not the general `CountMatchingRelativeTo`):
```rust
/// CR 205.3m / 508.1 / 613.1d: Count OTHER attacking creatures (any controller)
/// whose layer-resolved creature-type set shares ≥1 type with `relative_to`'s
/// layer-resolved creature-type set. `relative_to` resolves to the triggering
/// creature (Shared Animosity: the attacker that fired the "creature you control
/// attacks" trigger). Excludes `relative_to` itself ("other"). Returns 0 if
/// `relative_to` resolves to no object, is typeless, or combat is None.
OtherAttackersSharingCreatureType { relative_to: EffectTarget },
```
Rationale for minimal over general `CountMatchingRelativeTo { relative_to, filter, share_creature_type, .. }`: shared_animosity is the sole consumer; "all attackers, any controller, other, share-a-type" is its exact and complete semantics (ruling 2008-04-01: counts creatures not types, includes teammates' attackers). YAGNI + the PB-yield-overcount discipline argue against speculative config surface, and folding "shares a type" into `TargetFilter` would violate the "runtime relationship, not a `Characteristics` property — must be checked at every call site" contract that already bit éomer/Commissar. Changeling is handled for free: CR 702.73a materializes all creature types into the layer-resolved `subtypes` `OrdSet`, so set-intersection just works.

**×2 multiplier mechanism (goblin_piledriver "+2/+0 for each"):** use **`EffectAmount::Sum(AttackingCreatureCount{…}, AttackingCreatureCount{…})`** = 2×count. No new variant, no new multiplier primitive. `resolve_amount` handles `Sum(a,b) = resolve_amount(a)+resolve_amount(b)` (effects/mod.rs:7398); both sub-terms evaluate against the identical combat state at the same resolution instant → deterministic 2×count. This keeps PB-OS5 to a **single** new `EffectAmount` variant (matching the wire brief's singular framing) and avoids adding a `ScaledBy`/`Times` primitive that would have exactly one consumer today. A card-def comment makes the `Sum(x, x)` idiom's intent explicit. (Alternative `ScaledBy { amount, factor }` is noted under Risks as a future generalization if coefficient->1 pumps proliferate; deliberately NOT adopted now.)

**CDA arm:** the new variant is NOT CDA-eligible (resolving `relative_to` needs `EffectContext.triggering_creature_id`, absent in `resolve_cda_amount`). It only ever reaches the engine via the `Effect::ApplyContinuousEffect` → `ModifyPowerDynamic` → `resolve_amount` substitution (locked in at resolution, CR 608.2h/107.3f), never stored in a Layer-7a CDA. Add an **explicit** `=> 0` arm in `resolve_cda_amount` with a doc comment (mirroring the `CounterCountAtLastKnownInformation`/`SourcePowerAtLastKnownInformation` precedent) rather than relying on the `_` debug-assert catch-all — documents intent and prevents a debug panic if ever misauthored.

**Wire impact — confirmed SINGLE bump each:** adding one `EffectAmount` variant moves the SR-8 protocol fingerprint once (`PROTOCOL_VERSION 19→20`) and, per the "adding a variant is a variant-shape change → bump" convention, moves the hash schema once (`HASH_SCHEMA_VERSION 56→57`). New hash discriminant = **24** (current max is 23 = `ManaValueOfSacrificedCreature`). No other new variants → no additional bumps.

---

## CR rule text (abridged from MCP)

- **205.3m** — "Creatures and kindreds share their lists of subtypes; these subtypes are called creature types." (full list of ~300 types). Changeling (CR 702.73a) = "is every creature type" → all types materialize into the resolved subtype set.
- **508.1** — "First, the active player declares attackers. This turn-based action doesn't use the stack." (attackers live in `state.combat.attackers`, not a `Characteristics` field).
- **107.3f** — "Sometimes X appears in the text of a spell or ability but not in a … cost. If the value of X isn't defined, the controller … chooses the value of X at the appropriate time (either as it's put on the stack or as it resolves)." → the "for each" count is fixed at resolution.
- **613.1d / 613.3** — battlefield type/characteristic reads use layer-resolved characteristics; Changeling is a CDA in Layer 4.
- **Shared Animosity ruling (2008-04-01)** — "This ability counts creatures, not creature types." + "your teammate's attacking creatures are included in the calculation" → all-controller scope, count of creatures.
- **Goblin Piledriver ruling (2004-10-04)** — "The number of Goblins is counted when this ability resolves." → resolution-time count.
- **Muxus ruling (2020-06-23)** — "The bonus … is determined only as its last ability resolves." → resolution-time count.

---

## Engine Changes

### Change 1 — New `EffectAmount` variant
**File**: `crates/card-types/src/cards/card_definition.rs` (enum `EffectAmount`, after `TappedCreatureCount`/`HandSize`/`…SacrificedCreature` block, ~ line 2858+ / end of enum before closing brace at ~2862)
**Action**: Add
```rust
OtherAttackersSharingCreatureType { relative_to: EffectTarget },
```
with the doc comment from the Design section (cite CR 205.3m/508.1/613.1d, note "all attackers any controller, other, share-a-type, layer-resolved, Changeling-safe, discriminant 24 in state/hash.rs").
**Pattern**: Follow `AttackingCreatureCount` (same file, ~2843) and `ChosenTypeCreatureCount` (~2706) for the doc-comment style and the "reads combat/relative object, not a `Characteristics` field" caveat.

### Change 2 — `resolve_amount` executor (spell-effect path)
**File**: `crates/engine/src/effects/mod.rs` (the `resolve_amount` match; add arm alongside `AttackingCreatureCount` at ~7511 / before the closing `}` at ~7589 — this match is **exhaustive, no catch-all**, so the arm is compile-forced)
**Action**:
```rust
EffectAmount::OtherAttackersSharingCreatureType { relative_to } => {
    // Resolve relative_to (the triggering creature) to a single ObjectId.
    let Some(rel_id) = resolve_effect_target_list(state, relative_to, ctx)
        .into_iter()
        .find_map(|t| match t { ResolvedTarget::Object(id) => Some(id), _ => None })
        .or_else(|| if matches!(relative_to, EffectTarget::TriggeringCreature) {
            ctx.triggering_creature_id
        } else { None })
    else { return 0; };
    let Some(combat) = state.combat.as_ref() else { return 0; };
    // CR 205.3m/613.1d: relative creature's LAYER-RESOLVED subtypes (Changeling → all).
    let rel_subtypes = crate::rules::layers::expect_characteristics(state, rel_id).subtypes;
    if rel_subtypes.is_empty() { return 0; }
    state.objects.values().filter(|obj| {
        obj.zone == ZoneId::Battlefield
            && obj.is_phased_in()
            && obj.id != rel_id                       // CR 109.1/603.2: "other"
            && combat.is_attacking(obj.id)            // CR 508.1: all attackers, any controller
            && {
                let chars = crate::rules::layers::expect_characteristics(state, obj.id);
                chars.card_types.contains(&crate::state::types::CardType::Creature)
                    && chars.subtypes.iter().any(|st| rel_subtypes.contains(st)) // ≥1 shared type
            }
    }).count() as i32
}
```
**CR**: 205.3m (shared type via layer-resolved subtypes), 508.1 (attacking set, all controllers), 109.1/603.2 (exclude relative), 613.1d (layer-resolved reads).
**Notes for runner**: (a) confirm the exact spelling of `resolve_effect_target_list` and `ResolvedTarget` in scope (both used ~6475 / DealDamage arm ~311); (b) confirm `expect_characteristics` returns owned `Characteristics` (it does — cloned; `PowerOf` arm ~7132 uses it). `.subtypes` is an `imbl::OrdSet<SubType>` supporting `.contains`.

### Change 3 — `resolve_cda_amount` explicit non-CDA arm
**File**: `crates/engine/src/rules/layers.rs` (`resolve_cda_amount` match; add arm near the `CounterCountAtLastKnownInformation => 0` / `SourcePowerAtLastKnownInformation => 0` block at ~2038–2042, BEFORE the `_ =>` debug-assert catch-all at ~2117)
**Action**:
```rust
// CR 613: resolving `relative_to` (the triggering creature) requires EffectContext,
// which is absent here. This variant is only ever used via the spell-effect
// ApplyContinuousEffect -> ModifyPowerDynamic substitution (resolve_amount, value
// locked in at resolution per CR 608.2h/107.3f) and is never stored in a Layer-7a
// CDA. Returns 0 defensively (mirrors CounterCountAtLastKnownInformation).
EffectAmount::OtherAttackersSharingCreatureType { .. } => 0,
```
**Rationale**: keeps intent explicit and avoids the `_` debug_assert firing if a card is ever misauthored. Not a lockstep implementation because the variant is intentionally non-CDA — documented as such.

### Change 4 — Hash discriminant (compile-forced, wire)
**File**: `crates/engine/src/state/hash.rs` (`impl HashInto for EffectAmount`, exhaustive match ending at `ManaValueOfSacrificedCreature => 23` ~5455; **no catch-all** → compile-forced)
**Action**: add
```rust
// PB-OS5 (discriminant 24) — CR 205.3m/508.1: count of OTHER attacking creatures
// sharing a creature type with the relative (triggering) creature.
EffectAmount::OtherAttackersSharingCreatureType { relative_to } => {
    24u8.hash_into(hasher);
    relative_to.hash_into(hasher);
}
```
(Confirm `EffectTarget: HashInto` — it is; `PowerOf(EffectTarget)`/`ToughnessOf` already hash it.)

### Change 5 — Protocol version bump (SR-8), single 19→20
**File**: `crates/engine/src/rules/protocol.rs`
- `PROTOCOL_VERSION: u32 = 19` → `20` (line 178).
- Append a `- 20:` History doc line above `PROTOCOL_VERSION` (after the `- 19:` block ~171): "PB-OS5 (2026-07-19) — `EffectAmount` (already in the closure) gains `OtherAttackersSharingCreatureType { relative_to: EffectTarget }` (CR 205.3m/508.1 — count of other attacking creatures sharing a creature type with the triggering creature; OOS-EF4-1, Shared Animosity). Closure type count unchanged; `EffectAmount`'s declared shape moved, so the digest moves."
- Append a new `ProtocolEpoch { version: 20, fingerprint: <recomputed> }` row to `PROTOCOL_HISTORY` after the `version: 19` row (~359). **Never edit existing rows.**
- Update `PROTOCOL_SCHEMA_FINGERPRINT` (line 195) to the recomputed digest — **read the new value from the `tests/core/protocol_schema.rs` failure output**, set both it and the new history row to that value.

### Change 6 — Hash schema version bump, single 56→57
**File**: `crates/engine/src/state/hash.rs`
- `HASH_SCHEMA_VERSION: u8 = 56` → `57` (line 504).
- Append a `- N:` History doc line + a new `HashSchemaEpoch { version: 57, decl_fingerprint: <new>, stream_fingerprint: <new> }` row to `HASH_SCHEMA_HISTORY` (after the current tail ~566+). **Read both fingerprints from the `tests/core/hash_schema.rs` failure output.** Never edit existing rows.

### Change 7 — Machine-forced sentinel updates (exhaustive list)

| File | Site | Action |
|------|------|--------|
| `crates/engine/tests/core/protocol_schema.rs` | `PROTOCOL_VERSION, 19` (~872) + FROZEN prefix digest | bump to `20`; update frozen-prefix digest per its failure text |
| `crates/engine/tests/core/hash_schema.rs` | `HASH_SCHEMA_VERSION, 56u8` (1×) + FROZEN prefix + baseline pins | bump to `57`; update per failure text |
| `crates/engine/tests/primitives/pb_ef12_any_color_choice.rs` | `PROTOCOL_VERSION, 19` (363) | → `20` |
| `crates/engine/tests/primitives/pb_ef10_sacrifice_driven_amounts.rs` | `PROTOCOL_VERSION, 19` (1595) + `HASH_SCHEMA_VERSION, 56u8` (1600) | → `20` / `57` |
| `crates/engine/tests/primitives/pb_ef7_modal_activated.rs` | `PROTOCOL_VERSION, 19` (242) + `HASH_SCHEMA_VERSION, 56u8` (237) | → `20` / `57` |
| **All `HASH_SCHEMA_VERSION, 56` sentinels** | 37 occurrences across 36 files (see below) | global replace `HASH_SCHEMA_VERSION, 56` → `HASH_SCHEMA_VERSION, 57` |

`HASH_SCHEMA_VERSION, 56` sentinel files (run `Grep "HASH_SCHEMA_VERSION, 56"` to regenerate the live list before editing): `optional_cost_and_counter_tax.rs`, `effect_sacrifice_permanents_filter.rs`, `loyalty_target_validation.rs`, `core/hash_schema.rs`, and 32 files under `tests/primitives/` (`pbp_power_of_sacrificed_creature`, `primitive_pb_xa`, `primitive_pb_oos_lki_power_3`, `pb_ef2_create_token_recipient`, `pb_ef7_modal_activated`, `pb_ac6_phase_action_conditions`, `pbn_subtype_filtered_triggers`, `primitive_pb_xs`, `pb_ac8_restrictions_and_wingame`, `pb_ef10_sacrifice_driven_amounts`, `pb_ac5_alt_costs`, `primitive_pb_xa2`, `primitive_pb_eat`, `pb_ef11_wheel_greatest_discarded`, `primitive_pb_ewcd`, `pb_ef11_spell_single_target`, `primitive_pb_ewc`, `primitive_pb_ts`, `pb_ef6_target_opponent`, `pb_ef1_exclude_self_enforcement`, `pb_ac3_dynamic_pt_counts`, `pbt_up_to_n_targets` (2×), `primitive_pb_cc_a`, `primitive_pb_cc_c_followup`, `primitive_pb_xs_e`, `pb_ac7_type_change_ability_removal`, `primitive_pb_lki_cc`, `pb_ac9_wheel_and_misc`, `pb_ac4_per_mode_targeting`, `primitive_pb_lki_power`, `pbd_damaged_player_filter`, `pb_ac1_untap_counter`). `protocol_roundtrip.rs` uses relative `PROTOCOL_VERSION ± 1` — **no literal edit needed**.

**No other exhaustive `EffectAmount` match sites exist.** Verified: `tools/` has no Rust match on `EffectAmount` (only `authoring-report.py` string ref). `replay-viewer/view_model.rs` and `tui/stack_view.rs` match `StackObjectKind`/`KeywordAbility`, not `EffectAmount`. Still run `cargo build --workspace` after the impl phase to catch any missed exhaustive match (SR-8 gate).

---

## Card Definition Fixes / New Defs

### 1. `crates/card-defs/src/defs/shared_animosity.rs` — inert → **Complete**
**Oracle**: "Whenever a creature you control attacks, it gets +1/+0 until end of turn for each other attacking creature that shares a creature type with it."
**Current state**: `abilities: vec![]`, `Completeness::inert("OOS-EF4-1 …")`.
**Fix**: replace the TODO-only abilities with one `AbilityDefinition::Triggered`:
- `trigger_condition: WheneverCreatureYouControlAttacks { filter: None }`
- `effect: Effect::ApplyContinuousEffect { effect_def: ContinuousEffectDef { layer: PtModify, modification: LayerModification::ModifyPowerDynamic { amount: Box::new(EffectAmount::OtherAttackersSharingCreatureType { relative_to: EffectTarget::TriggeringCreature }), negate: false }, filter: EffectFilter::TriggeringCreature, duration: EffectDuration::UntilEndOfTurn, condition: None } }`
- `targets: vec![]`, `intervening_if: None`, `modes: None`, `trigger_zone: None`, `once_per_turn: false`.
Remove `completeness` (defaults to Complete) or set `Completeness::complete`. Cite CR 508.1m/205.3m/611.2a. Reference impl: `ogre_battledriver.rs` (TriggeringCreature pump shape).

### 2. `crates/card-defs/src/defs/goblin_piledriver.rs` — **NEW → Complete**
**Oracle**: "Protection from blue. Whenever this creature attacks, it gets +2/+0 until end of turn for each other attacking Goblin." 1/2 Goblin Warrior, `{1}{R}`.
**Sketch**:
- `types: creature_types(&["Goblin", "Warrior"])`, `power: Some(1), toughness: Some(2)`.
- Ability A: `AbilityDefinition::Keyword(KeywordAbility::ProtectionFrom(ProtectionQuality::FromColor(Color::Blue)))` (reference `akroma_angel_of_fury.rs`).
- Ability B: `Triggered { trigger_condition: WhenAttacks, effect: ApplyContinuousEffect { … ModifyPowerDynamic { amount: Box::new(EffectAmount::Sum( Box::new(goblin_count), Box::new(goblin_count) )), negate: false }, filter: EffectFilter::Source, duration: UntilEndOfTurn, condition: None }, … }` where
  `goblin_count = EffectAmount::AttackingCreatureCount { controller: PlayerTarget::EachPlayer, filter: Some(TargetFilter { has_subtype: Some(SubType("Goblin".into())), exclude_self: true, ..Default::default() }) }`.
- Add a comment: `// "+2/+0 per other attacking Goblin" = 2 × count, expressed as Sum(count, count); count excludes self via ctx.source (WhenAttacks → source is this creature).`
Completeness: Complete. Reference: `commissar_severina_raine.rs` (AttackingCreatureCount + EachPlayer + exclude_self + WhenAttacks).

### 3. `crates/card-defs/src/defs/goblin_rabblemaster.rs` — partial → **STAYS partial (pump clause implemented)** — *forced add via TODO sweep, not in original PB brief*
**Oracle**: "Other Goblin creatures you control attack each combat if able. At the beginning of combat on your turn, create a 1/1 red Goblin creature token with haste. Whenever Goblin Rabblemaster attacks, it gets +1/+0 until end of turn for each other attacking Goblin."
**Current state**: token trigger implemented; two TODOs (forced-attack restriction; "+1/+0 per attacking Goblin"). `Completeness::partial(...)`.
**Fix**: replace the `// TODO: "+1/+0 per attacking Goblin"` (line 54) with a `Triggered { WhenAttacks, ApplyContinuousEffect { ModifyPowerDynamic { amount: AttackingCreatureCount { controller: EachPlayer, filter: Some(has_subtype Goblin + exclude_self) }, negate: false }, filter: EffectFilter::Source, UntilEndOfTurn } }` — the **×1** (no `Sum`) analogue of piledriver.
**Correction to the existing completeness note**: it suggests `controller: Controller` — use `controller: EachPlayer` instead (CR-canonical "each other attacking Goblin", matching Commissar precedent; identical in normal single-attacker combat, EachPlayer is the safe reading).
**Stays partial**: the "Other Goblin creatures you control attack each combat if able" must-attack requirement has no `GameRestriction` variant (all existing variants are prohibitions). Update the completeness note to say the +1/+0 clause is now IMPLEMENTED and only the forced-attack clause remains blocked (name it: needs a subtype-filtered must-attack `GameRestriction` — out of PB-OS5 scope; file/keep as its own seed).

### 4. `crates/card-defs/src/defs/muxus_goblin_grandee.rs` — **NEW → partial**
**Oracle**: "When Muxus enters, reveal the top six cards of your library. Put all Goblin creature cards with mana value 5 or less from among them onto the battlefield and the rest on the bottom of your library in a random order. Whenever Muxus attacks, it gets +1/+1 until end of turn for each other Goblin you control." 4/4 Legendary Goblin Noble, `{4}{R}{R}`.
**Author**: the **attack half only**:
- `Triggered { WhenAttacks, ApplyContinuousEffect { modification: LayerModification::ModifyBothDynamic { amount: Box::new(EffectAmount::PermanentCount { filter: TargetFilter { controller: TargetController::You, has_subtype: Some(SubType("Goblin".into())), exclude_self: true, ..Default::default() }, controller: PlayerTarget::Controller }), negate: false }, filter: EffectFilter::Source, duration: UntilEndOfTurn, condition: None }, … }`.
- Note: **you-control, NOT attacking** (PermanentCount, not AttackingCreatureCount); creature not required by oracle ("other Goblin you control") but PermanentCount over Goblin-subtyped permanents is fine — Goblin is a creature type so only creatures carry it. exclude_self excludes Muxus via `ctx.source`.
**Stays partial**: the ETB half (reveal top six / put Goblin creature cards MV≤5 onto battlefield / rest to bottom in random order) is a reveal-and-put-from-library primitive tracked by **OOS-EF10 / PB-OS8** — DO NOT implement, DO NOT flip Complete. `Completeness::partial("PB-OS5: attack half authored (PermanentCount you-control + ModifyBothDynamic). ETB reveal-top-six/put-Goblins-onto-battlefield blocked on reveal-and-put-from-library primitive — OOS-EF10 / PB-OS8.")`. Leave the ETB as an honest comment, no gated stub.
**Justification for authoring a card that can't be Complete**: it is the you-control-scope execution vehicle for the mandatory 4-player scope decoy, and pre-stages the card for OS8. Tests use `GameStateBuilder` directly (no `validate_deck` gate), so the attack half is fully testable while partial.

---

## Unit Tests

**File**: `crates/engine/tests/primitives/pb_os5_relative_attacker_count.rs` (new; add `mod pb_os5_relative_attacker_count;` to `crates/engine/tests/primitives/main.rs` or the group mod file — SR-9a: never add a top-level `tests/*.rs`).

Drive attack triggers through the real `Command::DeclareAttackers` path (mirror `pb_ef4_triggering_creature_subject_source.rs` / `pb_ef3b_granted_keyword_triggers.rs`), then read layer-resolved power/toughness of the pumped creature via `calculate_characteristics` AFTER the trigger resolves.

**Tests to write**:
- `test_os5_shared_animosity_counts_shared_type_attackers` — Shared Animosity + 3 attackers you control sharing a type with the subject and 1 that shares none → subject gets `+3/+0`; a same-type creature that is **not attacking** does not count. CR 205.3m/508.1.
- `test_os5_shared_animosity_layer_resolved_subtype_decoy` (**mandatory layer-resolution decoy**) — an attacker whose creature type is granted by a Layer-4 effect (e.g. `AddSubtypes`/`AddAllCreatureTypes`/Changeling) shares with the subject and **is counted**; a base-typed creature sharing the type but **not attacking** is **not** counted. Proves the count reads layer-resolved subtypes, not base, and reads combat state. CR 613.1d/205.3m/702.73a.
- `test_os5_shared_animosity_excludes_triggering_creature` (**exclude-self, source≠subject**) — with only the subject attacking (no other same-type attackers) → `+0/+0`; proves "other" excludes the *triggering creature* (not the enchantment source). CR 109.1/603.2.
- `test_os5_piledriver_double_multiplier_and_exclude_self` (**mandatory piledriver exclude-self + ×2**) — Piledriver alone attacking → `+0/+0`; Piledriver + 1 other attacking Goblin → `+2/+0`; + 2 others → `+4/+0`. Proves `Sum(count,count)` = 2×, and exclude-self via `ctx.source`. CR 508.1.
- `test_os5_piledriver_ignores_nongoblin_attackers` — a non-Goblin attacker is not counted (fixed subtype filter). 
- `test_os5_muxus_you_control_scope_4player` (**mandatory 4-player scope decoy, you-control**) — 4-player game: Muxus attacks; you also control 2 non-attacking Goblins (counted) and an opponent controls a Goblin (attacking or not — **not** counted); Muxus itself excluded → `+2/+2`. Proves PermanentCount you-control scope (NOT all-controller, NOT attacking-only) + exclude-self. CR 205.3m.
- `test_os5_scope_animosity_piledriver_any_controller` (**scope decoy, all-controller**) — assert the animosity/piledriver counts do **not** filter by controller: a same-type / Goblin attacker NOT controlled by the trigger's controller would still count (construct via a controlled attacker of a different player if the harness permits; otherwise assert the code path uses `EachPlayer`/relative-set scope and add a negative assertion that a non-attacking opponent creature is excluded). Documents that scope is attacking-set membership, not controller.
- `test_os5_protocol_and_hash_sentinels` — `assert_eq!(PROTOCOL_VERSION, 20); assert_eq!(HASH_SCHEMA_VERSION, 57u8);` (own the sentinel in this PB's file, mirroring `pb_ef12`/`pb_ac3`).
- `test_os5_shared_animosity_registers` / `test_os5_piledriver_registers` — `all_cards()` contains them, defs load; piledriver/rabblemaster/muxus attack halves produce correct state through a real command path (**probe by execution, SR-34/36** — each flipped/authored card has an executing test path; do NOT source-trace).

**Pattern**: `pb_ac3_dynamic_pt_counts.rs` (AttackingCreatureCount + dynamic P/T through DeclareAttackers), `pb_ef4_triggering_creature_subject_source.rs` (TriggeringCreature pump), `mechanics_a_d/changeling.rs` (Changeling all-types materialization for the layer decoy).

---

## Verification Checklist

- [ ] `OtherAttackersSharingCreatureType` variant compiles (`cargo check -p mtg-card-defs` then `-p mtg-engine`)
- [ ] `resolve_amount` arm added (effects/mod.rs); `resolve_cda_amount` explicit `=> 0` arm added (layers.rs); `HashInto` arm added (hash.rs, discriminant 24)
- [ ] `PROTOCOL_VERSION 19→20`: fingerprint re-pinned + `PROTOCOL_HISTORY` row appended + `- 20:` history line; `tests/core/protocol_schema.rs` green
- [ ] `HASH_SCHEMA_VERSION 56→57`: two fingerprints re-pinned + `HASH_SCHEMA_HISTORY` row appended + history line; `tests/core/hash_schema.rs` green
- [ ] All `HASH_SCHEMA_VERSION, 56` (37×/36 files) and `PROTOCOL_VERSION, 19` (3 test files + protocol_schema) sentinels bumped
- [ ] shared_animosity → Complete; goblin_piledriver (new) → Complete; goblin_rabblemaster pump clause implemented, note updated, stays partial (forced-attack blocker named); muxus (new) attack half implemented, stays partial (ETB blocker named)
- [ ] New test file wired into the `primitives` group mod (SR-9a); all new tests pass
- [ ] `cargo build --workspace` (SR-8 exhaustive-match gate) + `cargo test --all` + `cargo clippy -- -D warnings` + `cargo fmt --check` + `tools/check-defs-fmt.sh` (SR-35)
- [ ] No remaining TODO in shared_animosity.rs / goblin_piledriver.rs; rabblemaster/muxus TODOs limited to the honestly-named out-of-scope blockers
- [ ] Close-out: banner+strike PB-OS5 in `oos-retriage-plan-2026-07-18.md` §3; seed banner OOS-EF4-1 in `ef-batch-plan-2026-07-17.md` §8; update `workstream-state.md`; reset `memory/primitive-wip.md` to IDLE

---

## Risks & Edge Cases

- **`Sum(x, x)` idiom obscurity** — a future reader may misread the doubled `AttackingCreatureCount` as a bug. Mitigated by an explicit card-def comment. If coefficient>1 "for each" pumps proliferate, a follow-up PB should introduce `EffectAmount::ScaledBy { amount: Box<EffectAmount>, factor: i32 }` (CDA-safe: `factor * resolve_cda_amount(inner)`) and migrate piledriver; deliberately deferred now (single consumer, YAGNI).
- **rabblemaster completeness note bug** — its pre-diagnosed fix says `controller: Controller`; the CR-correct scope is `EachPlayer` ("each other attacking Goblin", Commissar precedent). Immaterial in single-attacker combat but flag + correct the note.
- **relative_to fallback** — if the triggering creature left combat before the trigger resolves, `resolve_effect_target_list(TriggeringCreature)` may return empty; the arm falls back to raw `ctx.triggering_creature_id` (mirrors the DealDamage arm ~317) and returns 0 if still unresolved. Shared Animosity's UEOT pump on a departed subject is a no-op anyway (nothing to pump).
- **Typeless subject** — a subject with zero creature types (all subtypes stripped by a Layer-4 effect) yields count 0 (early `is_empty()` return). Correct per "shares a creature type."
- **Changeling double-edge** — a Changeling *subject* shares with every attacker (counts all other attackers); a Changeling *candidate* shares with any typed subject. Both are correct (CR 702.73a) and are why the layer-resolved decoy test matters.
- **Non-CDA misauthoring** — if a card author ever stores the variant in a Layer-7a CDA static ability, the explicit `=> 0` arm returns 0 (documented). Not silently wrong game state for the intended attack-trigger usage; only a defensive floor for misuse.
- **Wire single-bump discipline** — resist adding `ScaledBy` in the same PB; that would still be one protocol bump but two discriminants and expands review surface beyond the OOS-EF4-1 scope. Stop-and-flag if the fix phase tempts a second variant (conventions.md implement-phase default-to-defer).
- **muxus partial legitimacy** — reviewer may question authoring a never-Complete card; the plan justifies it as the you-control scope test vehicle + OS8 pre-stage. If the reviewer prefers, the you-control decoy can instead use a synthetic 2-line test card, and muxus creation deferred to OS8 — coordinator's call, but authoring it now is the recommended path per the PB brief.
