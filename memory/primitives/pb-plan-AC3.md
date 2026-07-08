# Primitive Batch Plan: PB-AC3 — Dynamic P/T & count amounts (CDA residual)

**Generated**: 2026-07-08
**Primitive**: Dynamic count amounts + a Layer-7b dynamic P/T *set* modification, plus
"CDA residual" card cleanup (backfill cards that were left with stale TODOs after the
dynamic-P/T primitives shipped in PB-CC-C / PB-CC-C-followup / PB-TS).
**CR Rules**: CR 613.4a/b/c (Layer 7 sublayers), CR 613.3 (CDA-first ordering), CR 107.3c/k
(X value locked on the stack), CR 608.2h (X locked in at resolution for one-shot effects),
CR 611.2c (continuous-effect membership re-evaluation), CR 122 (counters not layer-derived),
CR 508/509 (attackers/blockers), CR 111.1 (token creation count).
**Cards affected**: ~7 CLEAN (3 net-new-primitive + 4 CDA/token residual) + ~4 PARTIAL
that PB-AC3 unblocks only one clause of (stay blocked under W6 no-partials policy).
**Dependencies**: PB-CC-C (`ModifyBothDynamic`/`ModifyPowerDynamic`/`ModifyToughnessDynamic`),
PB-CC-C-followup (`CdaModifyPowerToughness`, live CDA re-eval), PB-28 (`SetPtDynamic`, `Sum`),
PB-TS (`TokenSpec.count: EffectAmount`), PB-XA/XA2 (combat/tap `TargetFilter` fields — used
only for the optional filter arg). ALL SHIPPED.
**Deferred items from prior PBs**: none directly assigned; this batch IS the deferred
"CDA residual" cleanup of stale dynamic-P/T TODOs.

---

## TL;DR — What is actually net-new vs already-shipped

The PB-AC3 brief (acceptance criterion 4225) names four primitives. Research against the
current tree shows **most already exist**; only a small, precise slice is net-new. Read this
section before implementing — it changes the scope materially.

| Brief item | Status | Reality |
|---|---|---|
| `LayerModification::ModifyBoth` accepting `EffectAmount` (CDA / 7a/7c) | **ALREADY SHIPPED** | `SetPtDynamic {power,toughness}` (Layer 7a CDA set) and `ModifyBothDynamic {amount,negate}` (Layer 7c modify) already exist. No new *CDA* variant needed. |
| Layer-7b **set** base P/T to a dynamic value (spell/ability, X-locked) | **NET-NEW** | No variant sets base P/T to a dynamic amount for a one-shot effect. This is the genuine gap. Ship `LayerModification::SetBothDynamic { amount }` (substituted to `SetPowerToughness` at resolution). Unblocks **Mirror Entity** (its TODO literally names `LayerModification::SetBothDynamic(EffectAmount)`). |
| `EffectAmount::AttackingCreatureCount` | **NET-NEW** | Not expressible: `PermanentCount` uses `matches_filter(&Characteristics)`, which cannot see combat state (attacking lives on `CombatState`, not `Characteristics`). |
| `EffectAmount::TappedCreatureCount` | **NET-NEW** | Not expressible: tapped state lives on `GameObject.status.tapped`, not `Characteristics`; `PermanentCount` can't read it. |
| `EffectAmount::HandSize` | **REDUNDANT — recommend SKIP** | Fully expressible today as `CardCount { zone: ZoneTarget::Hand { owner }, player, filter: None }`, handled in BOTH `resolve_amount` and `resolve_cda_amount`. Adding `HandSize` would be a dead alias. See "Decision: HandSize" below. |
| Power-based token count | **ALREADY SHIPPED** | `TokenSpec.count: EffectAmount` (PB-TS). Set `count: EffectAmount::PowerOf(EffectTarget::Source)`. Pure card fix (Krenko). No engine change. |

**Net-new engine surface = 2 `EffectAmount` variants + 1 `LayerModification` variant + a
`CombatState::is_attacking` helper.** Everything else is verification + card backfill.

### Decision: HandSize (coordinator input welcome, planner recommends SKIP)
Acceptance criterion 4225 lists `EffectAmount::HandSize`. It is 100% redundant with the
existing `CardCount { zone: ZoneTarget::Hand { owner: PlayerTarget::Controller }, .. }`,
which already resolves in both the spell-effect path and the CDA path. No roster card is
blocked *solely* on a missing HandSize; every "cards in hand" card can use `CardCount`.
Adding `HandSize` violates the codebase's discriminant-hygiene norm (no redundant variants).
**Recommendation: do NOT add `HandSize`; satisfy 4225 by documenting the `CardCount` mapping
and, if any roster card needs a Maro-style CDA, author it with `SetPtDynamic { power:
CardCount{Hand}, toughness: CardCount{Hand} }`.** If the coordinator insists on the literal
variant, add it as `HandSize { player: PlayerTarget }` (disc 21) delegating to the same
counting logic as `CardCount{Hand}` — but flag it as an alias in the doc-comment.

---

## CR Rule Text (confirmed via MCP)

**CR 613.4** — Within layer 7, sublayers apply in order; within each sublayer, timestamp
order (dependency may reorder):
- **613.4a Layer 7a**: effects from CDAs that define P/T (see 604.3).
- **613.4b Layer 7b**: effects that *set* P/T to a specific number/value. Effects referring
  to *base* P/T apply here.
- **613.4c Layer 7c**: effects and counters that *modify* P/T (don't set).
- **613.4d Layer 7d**: switch P/T.

**CR 613.3** — Within layers 2–6, CDAs apply first, then other effects in timestamp order.

**CR 107.3c** — If X is defined by the text of a spell/ability, that is X's value while on
the stack; controller doesn't choose. **107.3k** — an activated ability's X in its cost is
independent per activation. **107.3m** — an X-referencing spell's X carries to its ETB
ability/replacement.

These pin the sublayer choices below: `SetBothDynamic` is a **Layer 7b set of base P/T**
(Mirror Entity: "have base power and toughness X/X"), X locked at activation (CR 107.3k /
608.2h). `SetPtDynamic` (already shipped) is **Layer 7a CDA**. `ModifyBothDynamic` (shipped)
is **Layer 7c modify**.

---

## Engine Changes

### Change 1 — `EffectAmount::AttackingCreatureCount` + `TappedCreatureCount` (enum def)

**File**: `crates/engine/src/cards/card_definition.rs`
**Action**: Add two variants to `enum EffectAmount` (immediately after
`SourcePowerAtLastKnownInformation`, currently ending at **line ~2497/2501**, before the
closing `}`; note the existing `// TODO (OOS-LKI-Power-1)` comment sits just above the brace
— insert before it or after `SourcePowerAtLastKnownInformation`).

```rust
/// CR 508.1 / 509: Count creatures that are currently attacking, controlled by the
/// resolved player(s), optionally narrowed by `filter`. Reads `state.combat.attackers`
/// (a `CombatState` map keyed by attacker ObjectId) — NOT a `Characteristics` field —
/// so it is unavailable via `PermanentCount`. Returns 0 when `state.combat` is `None`
/// (outside combat) or no attackers match.
///
/// `controller = PlayerTarget::EachPlayer` gives the unrestricted "number of attacking
/// creatures" reading (Keep Watch, Mishra, Galadhrim Ambush — all attackers belong to the
/// active player in normal combat, but EachPlayer is the CR-correct "all" scope).
/// `controller = PlayerTarget::Controller` + `filter = Some(<subtype>)` gives
/// "attacking [Dragons] you control" (The Ur-Dragon — out of primary scope).
/// `filter` with `exclude_self: true` gives "other attacking creatures" (Commissar
/// Severina Raine); `ctx.source` is threaded for that check in the effect path.
///
/// Filter matching uses layer-resolved characteristics in the effect path
/// (`resolve_amount`) and BASE characteristics in the CDA path (`resolve_cda_amount`,
/// to avoid layer recursion). Phased-out permanents are excluded (CR 702.26d).
///
/// Discriminant 19 (state/hash.rs).
AttackingCreatureCount {
    controller: PlayerTarget,
    filter: Option<TargetFilter>,
},
/// CR 613 / status: Count creatures on the battlefield that are TAPPED
/// (`GameObject.status.tapped`), controlled by the resolved player(s), optionally
/// narrowed by `filter`. Tapped status is NOT a `Characteristics` field, so this is
/// unavailable via `PermanentCount`. Phased-out permanents excluded.
///
/// `controller = PlayerTarget::Controller`, `filter = None` gives "tapped creatures you
/// control" (Throne of the God-Pharaoh). Same base-vs-layer-resolved filter split as
/// `AttackingCreatureCount`.
///
/// Discriminant 20 (state/hash.rs).
TappedCreatureCount {
    controller: PlayerTarget,
    filter: Option<TargetFilter>,
},
```

**Note on optional filter**: `TargetFilter` is already in scope in `card_definition.rs` and
already has a `HashInto` impl (hash.rs:4489). The `filter` field lets one variant cover the
plain "attacking/tapped creatures" case (`None`) and the future filtered cases without a new
primitive. Both roster CLEAN cards (Keep Watch, Throne) use `filter: None`.

### Change 2 — `LayerModification::SetBothDynamic` (enum def)

**File**: `crates/engine/src/state/continuous_effect.rs`
**Action**: Add one variant to `enum LayerModification`, in the **Layer 7b** region, right
after `SetPowerToughness { power, toughness }` (**line ~366**) OR grouped with the other
`*Dynamic` variants after `ModifyToughnessDynamic` (**line ~441**). Place it near
`SetPowerToughness` and label it Layer 7b to make the sublayer obvious.

```rust
/// Layer 7b: SET base power and toughness to a single dynamic value (CR 613.4b).
///
/// **Spell/ability path (CR 608.2h, CR 107.3k)**: `Effect::ApplyContinuousEffect`
/// substitutes this into a concrete `SetPowerToughness { power: v, toughness: v }` at
/// resolution so X is *locked in* at resolution (mirrors the `ModifyBothDynamic ->
/// ModifyBoth` substitution). If `SetBothDynamic` reaches `apply_layer_modification`
/// unsubstituted it means substitution was skipped — the layer arm degrades to a live
/// `resolve_cda_amount` eval (correct only for CDA-safe amounts; `XValue` would read 0,
/// same documented residual behavior as `ModifyBothDynamic`).
///
/// Used by Mirror Entity: "{X}: Until end of turn, creatures you control have base power
/// and toughness X/X ...". Distinct from `SetPtDynamic` (Layer 7a CDA, live re-eval, for
/// static `*/*` creatures) and from `ModifyBothDynamic` (Layer 7c, +X/+X modify).
///
/// Boxed to avoid `large_enum_variant`.
SetBothDynamic {
    amount: Box<crate::cards::card_definition::EffectAmount>,
},
```

### Change 3 — Evaluation: `resolve_amount` (spell-effect path)

**File**: `crates/engine/src/effects/mod.rs` — inside `resolve_amount` (**fn at line 6342**),
add two match arms (place near `PermanentCount`, ~line 6457). Model on `PermanentCount`:

```rust
EffectAmount::AttackingCreatureCount { controller, filter } => {
    let players = resolve_player_target_list(state, controller, ctx);
    let Some(combat) = state.combat.as_ref() else { return 0 };
    state.objects.values()
        .filter(|obj| {
            obj.zone == ZoneId::Battlefield
                && obj.is_phased_in()
                && players.contains(&obj.controller)
                && combat.attackers.contains_key(&obj.id)
                && filter.as_ref().map(|f| {
                    let chars = crate::rules::layers::calculate_characteristics(state, obj.id)
                        .unwrap_or_else(|| obj.characteristics.clone());
                    matches_filter(&chars, f)
                        && check_chosen_subtype_filter(state, ctx, f, &chars)
                        && check_has_counter_type(obj, f)
                        && check_exclude_self(ctx, obj.id, f)   // exclude_self via ctx.source
                }).unwrap_or(true)
        })
        .count() as i32
}
EffectAmount::TappedCreatureCount { controller, filter } => {
    let players = resolve_player_target_list(state, controller, ctx);
    state.objects.values()
        .filter(|obj| {
            obj.zone == ZoneId::Battlefield
                && obj.is_phased_in()
                && players.contains(&obj.controller)
                && obj.status.tapped
                && {
                    let chars = crate::rules::layers::calculate_characteristics(state, obj.id)
                        .unwrap_or_else(|| obj.characteristics.clone());
                    // require Creature card type (both count "creatures")
                    chars.card_types.contains(&CardType::Creature)
                }
                && filter.as_ref().map(|f| {
                    let chars = crate::rules::layers::calculate_characteristics(state, obj.id)
                        .unwrap_or_else(|| obj.characteristics.clone());
                    matches_filter(&chars, f) && check_has_counter_type(obj, f)
                }).unwrap_or(true)
        })
        .count() as i32
}
```

**Runner notes**:
- `AttackingCreatureCount` should also require Creature type only if a non-creature could
  ever be an attacker — in practice only creatures attack, so `combat.attackers` membership
  is sufficient; do NOT add a redundant creature-type gate there (keeps it robust to
  animated-permanent attackers).
- `check_exclude_self` may not exist as a named helper. If `TargetFilter.exclude_self`
  enforcement is inlined elsewhere (PB-XS added it at validate sites), replicate the same
  `f.exclude_self && obj.id == ctx.source` guard inline rather than inventing a helper.
  Only needed if you implement the filtered/exclude-self path; for the CLEAN roster
  (filter=None) this branch is never taken.
- Verify the exact accessor names against the tree: `obj.status.tapped`
  (game_object.rs:684), `obj.is_phased_in()`, `state.combat: Option<CombatState>`,
  `combat.attackers: OrdMap<ObjectId, AttackTarget>` (combat.rs:30).

### Change 4 — Evaluation: `resolve_cda_amount` (CDA path — LOCKSTEP INVARIANT)

**File**: `crates/engine/src/rules/layers.rs` — inside `resolve_cda_amount` (**fn at line
1480**), add matching arms. **This is mandatory**: the codebase documents a CRITICAL
INVARIANT (card_definition.rs:2366) that every new count `EffectAmount` MUST be added to
BOTH `resolve_amount` and `resolve_cda_amount` or CDA values silently diverge.

Model on the existing CDA `PermanentCount` arm (layers.rs:1488) — **use BASE characteristics**
(`obj.characteristics`, NOT `calculate_characteristics`) for filter matching to avoid CDA
recursion. Combat/tap state are not layer-derived, so reading `state.combat.attackers` /
`obj.status.tapped` introduces no recursion:

```rust
EffectAmount::AttackingCreatureCount { controller: pt, filter } => {
    let players = resolve_cda_player_target(state, pt, controller);
    let Some(combat) = state.combat.as_ref() else { return 0 };
    state.objects.values().filter(|obj| {
        obj.zone == ZoneId::Battlefield && obj.is_phased_in()
            && players.contains(&obj.controller)
            && combat.attackers.contains_key(&obj.id)
            && filter.as_ref().map(|f|
                crate::effects::matches_filter(&obj.characteristics, f)
                && crate::effects::check_has_counter_type(obj, f)
            ).unwrap_or(true)
    }).count() as i32
}
EffectAmount::TappedCreatureCount { controller: pt, filter } => {
    let players = resolve_cda_player_target(state, pt, controller);
    state.objects.values().filter(|obj| {
        obj.zone == ZoneId::Battlefield && obj.is_phased_in()
            && players.contains(&obj.controller)
            && obj.status.tapped
            && obj.characteristics.card_types.contains(&CardType::Creature)
            && filter.as_ref().map(|f|
                crate::effects::matches_filter(&obj.characteristics, f)
                && crate::effects::check_has_counter_type(obj, f)
            ).unwrap_or(true)
    }).count() as i32
}
```

No roster card exercises the CDA path for these two variants, but implementing them keeps the
invariant satisfied and avoids a `debug_assert!(false)` panic if a future CDA card uses them.

### Change 5 — Substitution: `SetBothDynamic -> SetPowerToughness`

**File**: `crates/engine/src/effects/mod.rs` — in the `Effect::ApplyContinuousEffect`
resolution, the `match &effect_def.modification` block (**line 2833**), add an arm alongside
the existing `ModifyBothDynamic`/`ModifyPowerDynamic`/`ModifyToughnessDynamic` arms:

```rust
LM::SetBothDynamic { amount } => {
    let v = resolve_amount(state, amount, ctx);   // X locked in here (CR 608.2h/107.3k)
    LM::SetPowerToughness { power: v, toughness: v }
}
```

This match is non-exhaustive (`other => other.clone()`), so it compiles without the arm — but
the arm is REQUIRED for correctness (without it, `SetBothDynamic` reaches
`apply_layer_modification` unsubstituted and reads X as 0 via `resolve_cda_amount`).

### Change 6 — Layer apply: `SetBothDynamic` arm in `apply_layer_modification`

**File**: `crates/engine/src/rules/layers.rs` — `apply_layer_modification` (**fn at line 993**)
is **exhaustive**, so the new variant REQUIRES an arm even though the spell path substitutes
it away. Add near `SetPtDynamic`/`SetPowerToughness` (line ~1122–1142):

```rust
// Layer 7b: residual live-eval SET (spell path substitutes to SetPowerToughness before
// reaching here; this arm handles any direct/static registration). CR 613.4b.
LayerModification::SetBothDynamic { amount } => {
    let controller = state.objects.get(&object_id)
        .map(|o| o.controller).unwrap_or(crate::state::player::PlayerId(0));
    let v = resolve_cda_amount(state, amount, object_id, controller);
    chars.power = Some(v);
    chars.toughness = Some(v);
}
```

### Change 7 — `CombatState::is_attacking` helper

**File**: `crates/engine/src/state/combat.rs` — add next to `is_blocking` (**line 122**):

```rust
/// PB-AC3: Returns true if `id` is currently declared as an attacker
/// (CR 508.1 — keys into `CombatState.attackers`).
pub fn is_attacking(&self, id: ObjectId) -> bool {
    self.attackers.contains_key(&id)
}
```

Optional (the count arms can inline `combat.attackers.contains_key`); adding the helper keeps
parity with `is_blocking` and documents intent. Not load-bearing.

### Change 8 — hash.rs (HashInto + schema version + changelog)

**File**: `crates/engine/src/state/hash.rs`

1. **`impl HashInto for EffectAmount`** (match at **line 4678**, currently ends at
   `SourcePowerAtLastKnownInformation => 18u8` line 4761). Exhaustive — compiler forces new
   arms. Add:
   ```rust
   EffectAmount::AttackingCreatureCount { controller, filter } => {
       19u8.hash_into(hasher);
       controller.hash_into(hasher);
       filter.hash_into(hasher);
   }
   EffectAmount::TappedCreatureCount { controller, filter } => {
       20u8.hash_into(hasher);
       controller.hash_into(hasher);
       filter.hash_into(hasher);
   }
   ```
   (`PlayerTarget` HashInto at 4619; `Option<TargetFilter>` via generic `Option`/`TargetFilter`
   at 4489 — both exist.)

2. **`impl HashInto for LayerModification`** (match at **line 1545**). Exhaustive. Next free
   discriminant is **28** (0–27 are used). Add:
   ```rust
   LayerModification::SetBothDynamic { amount } => {
       28u8.hash_into(hasher);
       amount.hash_into(hasher);
   }
   ```

3. **`HASH_SCHEMA_VERSION`** (**line 217**): bump `29 -> 30`. Add a changelog block `/// - 30:
   PB-AC3 ...` describing the two `EffectAmount` variants (disc 19, 20) and the
   `LayerModification::SetBothDynamic` variant (disc 28), noting all fields deserialize
   normally (new enum variants; existing serialized states are unaffected — no `#[serde(default)]`
   needed because no existing struct gained a field).

4. **PRE-EXISTING DEFECT to fix (see Risks §1)**: `LayerModification` HashInto has a
   **discriminant collision at 26** — `RemoveSuperType` (line 1577: `26u8`) AND
   `ModifyPowerDynamic` (line 1668: `26u8`) both hash the prefix `26`. Recommend fixing in
   this batch: reassign `RemoveSuperType` to discriminant **29** (next free after 28) and note
   it in the changelog. This is a real determinism hazard (two distinct effects can collide).
   If the reviewer prefers to file it separately, at minimum DO NOT reuse 26 for the new
   variant (use 28 as specified).

### Change 9 — No new runtime struct fields

PB-AC3 adds **zero** new mutable fields to `GameState`/`GameObject`/`PlayerState`/
`StackObject`/`PendingTrigger`. All additions are enum variants hashed via the two exhaustive
`HashInto` matches above. The PB-AC1 "missed hash field" class of bug does not apply here —
but the runner must still land the exhaustive-match arms (compiler-enforced) and the schema
bump. No `reset_turn_state` changes.

### Exhaustive-match site inventory (complete)

| File | Match | Line | Action |
|---|---|---|---|
| `cards/card_definition.rs` | `enum EffectAmount` def | ~2497 | Add 2 variants (disc 19, 20) |
| `state/continuous_effect.rs` | `enum LayerModification` def | ~366/441 | Add `SetBothDynamic` (disc 28) |
| `state/hash.rs` | `HashInto for EffectAmount` | 4678 | Add 2 arms |
| `state/hash.rs` | `HashInto for LayerModification` | 1545 | Add 1 arm; fix 26-collision |
| `state/hash.rs` | `HASH_SCHEMA_VERSION` + changelog | 217 | 29 → 30 |
| `effects/mod.rs` | `resolve_amount` | 6342 | Add 2 arms (effect path) |
| `effects/mod.rs` | `ApplyContinuousEffect` substitution `match &effect_def.modification` | 2833 | Add `SetBothDynamic` arm (non-exhaustive, but required) |
| `rules/layers.rs` | `resolve_cda_amount` | 1480 | Add 2 arms (CDA path — LOCKSTEP) |
| `rules/layers.rs` | `apply_layer_modification` | 993 | Add `SetBothDynamic` arm (exhaustive) |
| `state/combat.rs` | `impl CombatState` | 122 | Add `is_attacking` helper (optional) |

**NOT affected** (verified): `tools/tui/src/play/panels/stack_view.rs` and
`tools/replay-viewer/src/view_model.rs` exhaustive matches are on `StackObjectKind` /
`KeywordAbility` — PB-AC3 adds neither, so no arms needed there. `crates/simulator` and
`tools/` contain no exhaustive `EffectAmount`/`LayerModification` matches (grep confirmed).
Still run `cargo build --workspace` after the impl phase.

**Discriminant chains untouched**: no new `KeywordAbility`, `AbilityDefinition`,
`StackObjectKind`, or `TriggerData` variants. The only discriminants in play are the two
internal `HashInto` numberings (EffectAmount → 19,20; LayerModification → 28, plus the
26-collision fix → 29).

---

## Card Definition Fixes

### Net-new-primitive CLEAN

#### keep_watch.rs — Keep Watch ({2}{U} Instant)
**Oracle**: "Draw a card for each attacking creature."
**Current**: `DrawCards { count: EffectAmount::Fixed(1) }` placeholder + TODO (line 4,16).
**Fix**: `count: EffectAmount::AttackingCreatureCount { controller: PlayerTarget::EachPlayer,
filter: None }`. Remove both TODOs.

#### throne_of_the_god_pharaoh.rs — Throne of the God-Pharaoh ({2} Legendary Artifact)
**Oracle**: "At the beginning of your end step, each opponent loses life equal to the number
of tapped creatures you control."
**Current**: empty `abilities: vec![]` with TODO (whole effect omitted).
**Fix**: add `AbilityDefinition::Triggered { trigger_condition:
TriggerCondition::AtBeginningOfYourEndStep, effect: Effect::LoseLife { player:
PlayerTarget::EachOpponent, amount: EffectAmount::TappedCreatureCount { controller:
PlayerTarget::Controller, filter: None } }, .. }`. The generic end-step CardDef sweep (B14,
gotchas-infra "Turn Action") fires `AtBeginningOfYourEndStep` triggers automatically. Remove TODOs.

#### mirror_entity.rs — Mirror Entity ({2}{W} Creature — Shapeshifter 1/1)
**Oracle**: "Changeling. {X}: Until end of turn, creatures you control have base power and
toughness X/X and gain all creature types."
**Current**: `Changeling` keyword + activated ability with `Effect::Nothing` + TODOs (lines
20–23). Note the TODO claiming `AddAllCreatureTypes` is missing is STALE — it exists
(continuous_effect.rs:296).
**Fix**: replace `Effect::Nothing` with a sequence of two `Effect::ApplyContinuousEffect`
(or one effect that registers both continuous effects):
- effect A: `filter: CreaturesYouControl`, `duration: UntilEndOfTurn`, `layer: Layer7b`,
  `modification: SetBothDynamic { amount: Box::new(EffectAmount::XValue) }`.
- effect B: `filter: CreaturesYouControl`, `duration: UntilEndOfTurn`, `layer: Layer4`,
  `modification: AddAllCreatureTypes`.
Cost stays `Cost::Mana(ManaCost { x_count: 1, .. })`; `x_value` threads through
`ActivateAbility` into `EffectContext` (already wired). Verify the exact
`Effect::ApplyContinuousEffect` DSL shape (`ContinuousEffectDef`) and `EffectFilter`/
`EffectLayer` enum names against an existing card that registers an until-EOT anthem
(e.g. grep `EffectDuration::UntilEndOfTurn` + `ApplyContinuousEffect` in defs). Remove TODOs.

### CDA / token residual backfill (already-shipped primitives — stale TODOs)

#### krenko_tin_street_kingpin.rs — power-based token count (PB-TS, already shipped)
**Oracle**: "Whenever Krenko attacks, put a +1/+1 counter on it, then create a number of 1/1
red Goblin tokens equal to Krenko's power."
**Current**: `TokenSpec { count: EffectAmount::Fixed(2), .. }` + TODO (line 29).
**Fix**: `count: EffectAmount::PowerOf(EffectTarget::Source)`. The `AddCounter` runs first in
the `Sequence`, and `resolve_amount(PowerOf)` uses `calculate_characteristics` for battlefield
objects, so the freshly-added counter is reflected (2 tokens turn 1, growing). Remove TODO.
**No engine change.**

#### ulvenwald_hydra.rs — `*/*` CDA = lands you control (SetPtDynamic, shipped)
**Oracle**: "Reach. P/T each equal to lands you control. ETB: may search a land, put onto
battlefield tapped, shuffle." **Fix**: `power: None, toughness: None`; add a CDA
`SetPtDynamic { power: PermanentCount(lands you control), toughness: <same> }` via the
`*/*`-creature idiom (grep an existing `SetPtDynamic` land-count card — check if
`ulvenwald_hydra` line 24 TODO has siblings already authored, e.g. Tarmogoyf-style). ETB
search uses `Effect::SearchLibrary` (exists). Reach keyword exists.

#### wight_of_the_reliquary.rs — CDA +1/+1 per creature card in graveyard (shipped)
**Oracle**: "Vigilance. +1/+1 for each creature card in your graveyard. {T}, Sacrifice another
creature: search a land onto battlefield tapped." **Fix**: base 2/2 + `CdaModifyPowerToughness`
(or `ModifyBothDynamic`) with `amount = CardCount { zone: Graveyard, filter: creature-card }`;
activated `{T}`+sacrifice-another-creature → SearchLibrary. Remove both TODOs (lines 7, 27).

#### storm_kiln_artist.rs — CDA +1/+0 per artifact (shipped) + Magecraft (exists)
**Oracle**: "+1/+0 for each artifact you control. Magecraft — cast/copy instant/sorcery:
create a Treasure." **Fix**: base 2/2 + `CdaModifyPowerToughness { power:
Some(PermanentCount(artifacts you control)), toughness: None }`; Magecraft trigger already
supported (grep confirms `Magecraft` in storm_kiln_artist.rs + archmage_emeritus.rs);
Treasure token exists. Remove TODOs (lines 5, 22).

### PARTIAL — PB-AC3 unblocks ONE clause; card stays blocked under W6 no-partials policy

Author these only if the SECOND clause is also expressible (verify at authoring time). Under
the W6 "no partial implementations, no wrong game state" policy, if the second clause is a
genuine DSL gap, LEAVE the card blocked and update its TODO to name the *remaining* gap (so it
is not mis-scored as clean by `authoring-report.py`).

| Card | PB-AC3 unblocks | Remaining gap | Disposition |
|---|---|---|---|
| `galadhrim_ambush.rs` | token count = `AttackingCreatureCount` | "prevent all combat damage this turn by non-Elf creatures" — filtered prevention shield (DSL gap) | BLOCKED — update TODO to prevention gap |
| `mishra_claimed_by_gix.rs` | life-drain X = `AttackingCreatureCount` (via `DrainLife`) | Meld (keyword; not implemented) | BLOCKED — Meld |
| `ashaya_soul_of_the_wild.rs` | `*/*` CDA = lands (`SetPtDynamic`) | "Nontoken creatures you control are Forest lands in addition to their other types" — mass Layer-4 type-add static with filter | BLOCKED — Layer-4 static type-grant gap |
| `multani_yavimayas_avatar.rs` | CDA +1/+1 per land + per land in GY (`CdaModifyPowerToughness{ amount: Sum(...) }`) | "{1}{G}, Return two lands you control: return this from GY to hand" — graveyard-zone activated ability with return-N-lands additional cost | verify `ActivationZone::Graveyard` + return-lands cost; if gap, BLOCKED |

### Out of PB-AC3 scope (need a different primitive)

- `nighthawk_scavenger.rs`, `nethergoyf.rs` — CDA = card types in graveyard; needs
  `EffectAmount::CardTypesInGraveyard` (NOT in this batch). File as OOS seed.
- `harvest_season.rs`, `grand_warlord_radha.rs` — add mana = tapped/attacking creature count;
  needs a dynamic-mana-amount primitive (`AddMana` with `EffectAmount`). OOS seed.
- `the_ur_dragon.rs` — "attacking Dragons you control" (filtered attack count is expressible
  via `AttackingCreatureCount { controller: Controller, filter: Some(Dragon) }`) BUT also has
  eminence cost reduction + reflexive draw-that-many; verify whole card, likely PARTIAL.
- `commissar_severina_raine.rs` — "other attacking creatures" expressible via
  `AttackingCreatureCount { filter: exclude_self }`; verify the rest of the card before
  authoring.
- `promise_of_power.rs` — Demon token with P/T = cards in hand; needs token-attached CDA
  (`SetPtDynamic { CardCount{Hand} }` on the token spec) — authoring complexity, not a missing
  primitive. Consider if token-CDA authoring is in scope; otherwise OOS.

**TODO sweep assertion**: the grep
`(TODO|ENGINE-BLOCKED).*(attacking|tapped|power|hand|CDA|dynamic P/T|X/X)` over
`crates/engine/src/cards/defs/` returned the full candidate set above; every card citing
`AttackingCreatureCount`/`TappedCreatureCount`/`SetBothDynamic`/power-based-token/CDA-P-T is
accounted for in one of the four buckets. No card that names these primitives was dropped.

---

## Unit Tests

**File**: `crates/engine/tests/pb_ac3_dynamic_pt_counts.rs` (new). Pattern: follow existing
layer/CDA tests (grep `SetPtDynamic` / `ModifyBothDynamic` in `crates/engine/tests/`) and
`resolve_amount` count tests. Every test cites a CR section.

Count-amount tests (resolve_amount path):
- `test_attacking_creature_count_basic` — CR 508.1 / 613.1d. 4-player state, declare 2
  attackers; resolve `DrawCards { AttackingCreatureCount { EachPlayer, None } }`; assert 2
  cards drawn. (Keep Watch mechanic.)
- `test_attacking_creature_count_zero_outside_combat` — `state.combat = None` → 0 (negative).
- `test_attacking_creature_count_ignores_nonattacking` — creatures on battlefield not declared
  as attackers are not counted.
- `test_tapped_creature_count_basic` — CR 613 / status.tapped. Controller has 3 creatures, tap
  2; `LoseLife { EachOpponent, TappedCreatureCount { Controller, None } }`; assert each
  opponent lost 2. (Throne mechanic.)
- `test_tapped_creature_count_controller_scope` — opponent's tapped creatures are NOT counted
  when `controller = Controller`.
- `test_tapped_creature_count_excludes_phased_out` — CR 702.26d.

SetBothDynamic layer tests:
- `test_set_both_dynamic_sets_base_pt` — CR 613.4b. Activate Mirror-Entity-style `{X=3}` on
  a board of 1/1s; assert each of controller's creatures is base 3/3.
- `test_set_both_dynamic_locked_at_resolution` — CR 608.2h / 107.3k. After activation with
  X=3, a creature that ENTERS later still reads 3/3 (membership re-evaluated per CR 611.2c,
  value locked); changing board state does not change the 3.
- `test_set_both_dynamic_then_counter_layer_order` — CR 613.4b vs 613.4c. Base set to 3/3
  (7b), then a +1/+1 counter (7c) → 4/4. Asserts sublayer ordering (set before modify).
- `test_set_both_dynamic_then_anthem` — CR 613.4c. SetBothDynamic 3/3 + external anthem
  +1/+1 → 4/4.
- `test_set_both_dynamic_with_all_creature_types` — Mirror Entity full: creatures become 3/3
  AND every creature type (CR 205.3m / AddAllCreatureTypes). Verifies both continuous effects
  coexist and expire together (until EOT).

Power-based token test:
- `test_power_based_token_count` — CR 111.1 / PowerOf. Krenko attacks: AddCounter → power 2 →
  `CreateToken { count: PowerOf(Source) }` makes 2 tokens; second attack → power 3 → 3 tokens.

Integration tests using card defs (grep how existing card-def integration tests build state):
- `test_keep_watch_draws_per_attacker`
- `test_throne_end_step_drains_per_tapped_creature`
- `test_krenko_tokens_equal_power`
- `test_mirror_entity_pumps_and_types` (if activated-X ability wiring supports it in tests)

Determinism: existing hash tests cover the schema; add a spot check that a state containing a
`SetBothDynamic`/`AttackingCreatureCount` effect hashes deterministically across two identical
builds (mirror any existing `HashInto` round-trip test).

---

## Verification Checklist

- [ ] `EffectAmount` gains `AttackingCreatureCount` + `TappedCreatureCount`; compiles
- [ ] `LayerModification` gains `SetBothDynamic`; compiles
- [ ] Both count variants added to BOTH `resolve_amount` AND `resolve_cda_amount` (lockstep)
- [ ] `SetBothDynamic` substitution arm added in `effects/mod.rs` AND apply arm in `layers.rs`
- [ ] hash.rs: 3 new `HashInto` arms; `HASH_SCHEMA_VERSION` 29 → 30; changelog entry added
- [ ] hash.rs: `RemoveSuperType` 26-collision resolved (→ disc 29) OR explicitly deferred with note
- [ ] `CombatState::is_attacking` helper added (optional)
- [ ] Card fixes: keep_watch, throne, mirror_entity, krenko, ulvenwald_hydra, wight,
      storm_kiln_artist — TODOs removed
- [ ] PARTIAL/OOS cards: TODOs UPDATED to name the *remaining* gap (not removed, not
      mis-authored as clean)
- [ ] `cargo test --all` green
- [ ] `cargo clippy --all-targets -- -D warnings` clean (watch `large_enum_variant` — Box used)
- [ ] `cargo build --workspace` (TUI + replay-viewer compile; no new SOK/KW arms expected)
- [ ] `cargo fmt --check`
- [ ] `python3 tools/authoring-report.py` — post clean-coverage delta as task comment

---

## Risks & Edge Cases

1. **PRE-EXISTING HASH COLLISION (LayerModification disc 26)**: `RemoveSuperType` and
   `ModifyPowerDynamic` both hash prefix `26` (hash.rs:1577 vs 1668). Two distinct effects can
   produce colliding state hashes → distributed-verification divergence. Fix by reassigning
   `RemoveSuperType` → 29 in this batch (cheap, schema bump already happening). If deferred,
   file a LOW and DO NOT compound it — the new `SetBothDynamic` MUST use 28.

2. **Lockstep divergence (CR invariant)**: forgetting the `resolve_cda_amount` arms makes CDA
   evaluations of the new counts return 0 (via the `_ => debug_assert!` fallback) while the
   spell path returns the real count. Silent wrong game state. The checklist gate is explicit.

3. **SetBothDynamic substitution vs residual**: if the `effects/mod.rs` substitution arm is
   omitted, `SetBothDynamic { XValue }` reaches `apply_layer_modification` and
   `resolve_cda_amount(XValue)` returns 0 → Mirror Entity sets everything to 0/0. Both the
   substitution arm AND the apply arm are required; the substitution is the correctness path,
   the apply arm is the exhaustive-match/defensive path.

4. **Layer sublayer correctness**: `SetBothDynamic` MUST be registered at `EffectLayer::Layer7b`
   (base set), NOT 7a/7c. Mirror Entity says "base power and toughness" (7b). A +1/+1 counter
   or anthem must still stack on top (7c) → the `test_set_both_dynamic_then_counter_layer_order`
   test guards this. Registering at the wrong layer silently reorders against counters/anthems.

5. **W3-LC discipline**: the effect-path count arms read battlefield P/T-adjacent state via
   `calculate_characteristics` for the optional filter — correct. Do NOT read raw
   `obj.characteristics.power/toughness` at battlefield anywhere. The CDA-path arms
   deliberately use base chars to avoid recursion (documented, matches existing PermanentCount
   CDA arm) — this is the sanctioned exception, not a W3-LC violation.

6. **Krenko ordering**: `PowerOf(Source)` after `AddCounter` in the same `Sequence` must see
   the counter. Verified: `resolve_amount(PowerOf)` calls `calculate_characteristics`, which
   applies the just-added counter at Layer 7c. The token count grows each attack — the test
   asserts this.

7. **`*/*` authoring gotcha**: Ulvenwald Hydra / Ashaya use `power: None, toughness: None`
   (NOT `Some(0)`) with the CDA supplying values (gotchas-infra). Krenko/Throne are printed
   P/T, unaffected.

8. **Yield calibration** (`feedback_pb_yield_calibration.md`): brief estimated ~14; realistic
   CLEAN is ~7 (3 net-new-primitive + 4 residual). PARTIAL cards (Galadhrim, Mishra, Ashaya,
   Multani) each carry a SECOND clause that is a separate DSL gap — under W6 no-partials they
   stay blocked. Do not inflate the delta by authoring wrong-game-state halves.

9. **HandSize acceptance-criteria tension** (see TL;DR "Decision: HandSize"): the literal
   variant in criterion 4225 is redundant with `CardCount{Hand}`. Planner recommends SKIP and
   documenting the mapping. Surface this to the coordinator at review; if they want the alias,
   add `HandSize { player }` (disc 21) delegating to the CardCount logic.

---

## Summary (for coordinator)

- **Confirmed net-new primitives (3)**: `EffectAmount::AttackingCreatureCount` (disc 19),
  `EffectAmount::TappedCreatureCount` (disc 20), `LayerModification::SetBothDynamic` (disc 28,
  Layer 7b, substituted at resolution). Plus a `CombatState::is_attacking` helper.
- **Already-shipped (verify + backfill, no engine change)**: the CDA "ModifyBoth" family
  (`SetPtDynamic` 7a, `ModifyBothDynamic` 7c), power-based token count
  (`TokenSpec.count: EffectAmount::PowerOf`), and `HandSize` (= `CardCount{Hand}`).
- **Roster**: ~7 CLEAN — net-new (Keep Watch, Throne of the God-Pharaoh, Mirror Entity) +
  residual (Krenko Tin Street Kingpin, Ulvenwald Hydra, Wight of the Reliquary, Storm-Kiln
  Artist). ~4 PARTIAL (Galadhrim Ambush, Mishra, Ashaya, Multani) stay blocked on a second
  clause. Several OOS seeds (CardTypesInGraveyard, dynamic-mana-amount).
- **Key risks**: pre-existing LayerModification hash collision at disc 26 (fix to 29); the
  BOTH-paths lockstep invariant for the new counts; SetBothDynamic must be Layer 7b and MUST
  be substituted at resolution; HandSize acceptance-criteria redundancy (recommend SKIP).
