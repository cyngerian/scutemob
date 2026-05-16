# Primitive Batch Plan: PB-LS6 — Loyalty-target validation + Sorin reanimate-rider + Tamiyo freeze-rider

**Generated**: 2026-05-15
**Task**: scutemob-36 (LS-6 of the LOW-sweep campaign)
**Primitive**: Three independent LOW-issue engine unblocks bundled as one PB-scale batch:
  1. PB-T-L01 — thread `TargetRequirement` validation through `handle_activate_loyalty_ability`.
  2. PB-T-L02 — new `Effect::DestroyAndReanimate` variant (destroy targets, return the
     resulting graveyard cards to the battlefield under controller's control).
  3. PB-T-L03 — new per-object `skip_untap_steps: u32` counter on `GameObject` +
     `Effect::PreventNextUntap` variant ("permanent doesn't untap during its controller's
     next untap step").
**CR Rules**: 606 (loyalty abilities), 601.2c (target announcement/validation),
  502.3 (untap step — effects can keep permanents from untapping), 400.7 (object identity),
  701.7 (destroy), 614 (replacement effects), 603.6a (ETB triggers).
**Cards affected**: 8 total (6 loyalty-validation card defs already correct — they gain
  enforcement only; 2 card-def TODO fixes — Sorin & Tamiyo; +1 forced add from TODO sweep —
  Hands of Binding).
**Dependencies**: none — all required primitives (`MoveZone`, `ReturnAllFromGraveyardToBattlefield`,
  `validate_targets_with_source`, the EOC-flag pattern) already exist.
**Deferred items from prior PBs**: PB-T-L01/L02/L03 are themselves the deferred items from
  the PB-T batch (see `docs/mtg-engine-low-issues-remediation.md` PB-T section).

---

## TODO Sweep (roster-recall gate)

Ran `Grep "TODO.*[Uu]ntap | TODO.*[Rr]eanimate | TODO.*loyalty.*target"` across
`crates/engine/src/cards/defs/`. Findings relevant to this batch:

- **`hands_of_binding.rs`** — FORCED ADD for L03. TODO at lines 6-8 + 21 explicitly names
  the missing primitive: *"no `Effect::PreventNextUntap` or `EffectDuration::UntilNextUntapStep`...
  until a `DoesntUntapNextTurn` effect primitive exists"*. Hands of Binding's spell effect
  is "Tap target creature an opponent controls. That creature doesn't untap during its
  controller's next untap step." — identical rider to Tamiyo -2. The new `Effect::PreventNextUntap`
  unblocks it directly. *Added via pre-existing TODO sweep — not in original PB brief.*
- **`puppeteer_clique.rs`** — NOT a forced add. TODO names a *targeted ETB reanimate from an
  opponent's graveyard*, a different shape (single declared target into the graveyard zone)
  than Sorin's destroy-then-reanimate. `Effect::DestroyAndReanimate` does not unblock it.
  Recorded here for completeness; leave its TODO in place.
- Many other `TODO.*untap` hits (`mana_vault`, `goblin_sharpshooter`, `seedborn_muse`,
  `najeela`, `sky_hussar`, `benefactors_draught`, `cloud_of_faeries`, `snap`, `rewind`,
  `quest_for_renewal`, etc.) reference *different* untap primitives — static "doesn't untap
  during your untap step", `Effect::UntapAll`, multi-target untap-N, untap triggers. None
  are unblocked by `Effect::PreventNextUntap` (which is specifically a one-shot "skip the
  *next* untap step"). Out of scope; leave as-is.

**TODO sweep result: 1 forced add (`hands_of_binding.rs`).**

---

## CR Rule Text

**CR 606 — Loyalty abilities** (full):
- 606.1. Some activated abilities are loyalty abilities, subject to special rules.
- 606.2. An activated ability with a loyalty symbol in its cost is a loyalty ability.
- 606.3. A player may activate a loyalty ability of a permanent they control any time they
  have priority and the stack is empty during a main phase of their turn, but only if no
  player has previously activated a loyalty ability of that permanent that turn.
- 606.4. The cost is to put on / remove loyalty counters as shown by the loyalty symbol.
- 606.5. Multiple add/remove costs combine into a single cost.
- 606.6. A loyalty ability with a negative cost can't be activated unless the permanent has
  at least that many loyalty counters.

**CR 601.2c** (target announcement — applies to activated/loyalty abilities via CR 602.2b):
"The player announces their choice of an appropriate object or player for each target...
If the spell has a variable number of targets, the player announces how many targets they
will choose... The same target can't be chosen multiple times for any one instance of the
word 'target'... The chosen objects and/or players each become a target."

**CR 502.3 — Untap step**: "the active player determines which permanents they control will
untap. Then they untap them all simultaneously. This turn-based action doesn't use the stack.
Normally, all of a player's permanents untap, but **effects can keep one or more of a player's
permanents from untapping.**"

(Note: the Tamiyo card-def comment cites "CR 613.6" for the freeze rider. That is incorrect —
"doesn't untap" is NOT a continuous/layer effect. It is an untap-step exception under CR 502.3.
The implementation models it as a per-object counter consumed at the untap step, not a
`ContinuousEffect`. Fix the comment when editing the card def.)

---

## Issue 1 — PB-T-L01: Loyalty-ability target validation

### Analysis

`handle_activate_loyalty_ability` (`crates/engine/src/rules/engine.rs:2224-2402`) converts
`Vec<Target>` → `Vec<SpellTarget>` (lines 2326-2341) with **no validation whatsoever**. It
never calls `validate_targets` / `validate_targets_with_source` /
`validate_object_satisfies_requirement`. A player can activate Sorin's -6 targeting a land,
Basri's +1 targeting an opponent's creature when it requires "you control", etc.

The fix mirrors the activated-ability path in `rules/abilities.rs:328-352`
(`handle_activate_ability`), which already does exactly this for non-loyalty activated
abilities. That code:
1. Computes `source_chars` via `calculate_characteristics(state, source)` with a fallback
   to base characteristics.
2. Calls `validate_targets_with_source(state, &targets, &target_requirements, player,
   source_chars.as_ref(), source)` — but only `if !target_requirements.is_empty()`.

The loyalty handler already extracts the ability via the `LoyaltyAbility { cost, effect, .. }`
destructure at engine.rs:2292. The `targets` field of `AbilityDefinition::LoyaltyAbility` is
currently discarded by the `..` — it must be bound.

### Engine Changes

**File**: `crates/engine/src/rules/engine.rs`

**Change 1a — bind the ability's `targets`** (line 2292):
Replace
```rust
let AbilityDefinition::LoyaltyAbility { cost, effect, .. } = ability else {
```
with
```rust
let AbilityDefinition::LoyaltyAbility { cost, effect, targets: ability_targets } = ability else {
```
`ability` is `&AbilityDefinition`, so `ability_targets` is `&Vec<TargetRequirement>`. Clone it
immediately (`let ability_targets = ability_targets.clone();`) because `def` (the
`Arc<CardDefinition>` borrow via `state.card_registry.get`) must be dropped before the
mutable `state.objects.get_mut` at line 2317. Note: `cost` and `effect` are already cloned
downstream (`effective_cost` computed from `cost`, `effect_clone = effect.clone()`), so the
existing borrow lifetime already ends before the mutation — but cloning `ability_targets` up
front is the safe pattern and matches `handle_activate_ability`.

**Change 1b — validate before paying the loyalty cost** (insert after line 2315, the CR 606.6
loyalty-sufficiency check, and BEFORE line 2316 "Pay the loyalty cost"):
```rust
// CR 601.2c: validate declared targets against the ability's TargetRequirements
// BEFORE paying the loyalty cost, so an illegal activation doesn't burn loyalty.
// Mirrors the activated-ability path in rules/abilities.rs:328-352.
if !ability_targets.is_empty() {
    let source_chars = crate::rules::layers::calculate_characteristics(state, source)
        .or_else(|| state.objects.get(&source).map(|o| o.characteristics.clone()));
    crate::rules::casting::validate_targets_with_source(
        state,
        &targets,
        &ability_targets,
        player,
        source_chars.as_ref(),
        source,
    )?;
}
```
Placement rationale: all the CR 606.3 timing checks and the CR 606.6 loyalty-sufficiency
check already run before this point and return `Err` early. Target validation must also run
before the `obj.counters.insert(CounterType::Loyalty, ...)` mutation at line 2317-2322 so a
rejected activation leaves loyalty untouched (parity with "mana not wasted on illegal
activations" in the activated-ability path).

`validate_targets_with_source` returns `Result<Vec<SpellTarget>, GameStateError>`; we
discard the `Ok` value here (the handler builds its own `spell_targets` at line 2326 with
zone snapshots). Using `?` propagates `InvalidTarget` / `InvalidCommand` errors to the
caller. This is acceptable — the existing `spell_targets` construction is a strict superset
of validation's zone capture, and changing it to consume validation's output is a larger
refactor than this LOW warrants. (Optional cleanup, NOT required: replace lines 2326-2341
with the validated `Vec<SpellTarget>` — defer; out of scope.)

**Imports**: `TargetRequirement` is already reachable — `AbilityDefinition` is imported at
engine.rs:2232 (`use crate::cards::card_definition::{AbilityDefinition, LoyaltyCost};`).
`validate_targets_with_source` is `pub(crate)` in `casting.rs` and called fully-qualified, so
no new `use` is needed.

### Card defs affected (enforcement only — no edits)

These 6 loyalty cards listed in the PB-T-L01 brief already declare correct
`TargetRequirement`s; they currently get NO validation and this change makes their existing
declarations enforced. No card-def file edits required for L01 — verify each compiles and the
declared requirements are sane:
- `sorin_lord_of_innistrad.rs` — -6: `UpToN { count: 3, inner: TargetPermanentWithFilter
  { has_card_types: [Creature, Planeswalker] } }`. ✓
- `basri_ket.rs` — verify +1/-2 target requirements.
- `tamiyo_field_researcher.rs` — -2: `UpToN { count: 2, inner: TargetPermanentWithFilter
  { non_land: true } }`. ✓ (also edited by L03 below).
- `teferi_temporal_archmage.rs`
- `tyvar_jubilant_brawler.rs`
- `tyvar_kell.rs`

Action for the runner: open each, confirm the loyalty abilities that target have a non-empty
`targets: vec![...]`, and that the filter matches the oracle text. Do NOT change them unless
a filter is provably wrong — that would be a separate card-def fix.

### Risks

- **Up-to-N with 0 declared targets**: `target_count_range` gives `min=0` for `UpToN`, so
  declaring zero targets is legal — `validate_targets_inner` accepts an empty `targets`
  slice when `min_t == 0`. Sorin -6 / Tamiyo -2 / Basri +1 with no targets must still
  succeed. The `if !ability_targets.is_empty()` guard is on the *requirement* list, not the
  declared-target list, so a card that declares `UpToN` requirements will still run
  validation with 0 declared targets — correct.
- **Loyalty abilities with `targets: vec![]`** (Sorin +1, Sorin -2, Tamiyo +1, Tamiyo -7):
  the guard skips validation entirely. Correct — no targets to validate.

---

## Issue 2 — PB-T-L02: Sorin -6 destroy-then-reanimate rider

### Oracle text (confirmed via MCP)

Sorin, Lord of Innistrad −6: "Destroy up to three target creatures and/or other
planeswalkers. **Return each card put into a graveyard this way to the battlefield under
your control.**"

Rulings of note:
- Token creatures can be targeted/destroyed but won't return (they cease to exist as an SBA
  before reanimation — CR 704.5d).
- A creature with undying destroyed this way: undying triggers but the creature is already
  returned by Sorin, so undying's trigger does nothing on resolution.
- Sorin's -6 can return cards from **opponents'** graveyards (the cards it just destroyed) —
  reanimation must NOT be restricted to the controller's own graveyard.

### Design decision: new `Effect` variant (reuse considered, rejected)

**Reuse analysis**:
- `Effect::ReturnAllFromGraveyardToBattlefield` returns *all matching cards already in
  graveyards* — it cannot scope to "the specific cards this effect just destroyed". Wrong.
- A `Sequence([DestroyPermanent x3, <reanimate>])` cannot work: after `DestroyPermanent`
  moves a creature to the graveyard it becomes a NEW object (CR 400.7) with a new ObjectId
  not tracked in `ctx.target_remaps` (DestroyPermanent does not write remaps), and
  `EffectContext` has no field carrying "objects put into graveyards by the last effect".
  Threading a new context field + a `ReanimateLastDestroyed` effect would be two new
  primitives and a context-field hash bump — strictly more surface than one self-contained
  variant.
- `MoveZone` targeting an opponent's graveyard cannot be sequenced either, same ObjectId
  problem.

**Decision**: add ONE new self-contained variant, `Effect::DestroyAndReanimate`, that
destroys the declared targets and reanimates the resulting graveyard cards in a single
atomic effect. This is the minimal surface and keeps the new-ObjectId tracking internal.

### Engine Changes

**Change 2a — new `Effect` variant**

**File**: `crates/engine/src/cards/card_definition.rs`
**Action**: add after `Effect::DestroyAll { .. }` (around line 1294), in the same
"destroy" cluster:
```rust
/// CR 701.7 + reanimation: destroy each declared target permanent, then return every
/// card actually put into a graveyard by this destruction to the battlefield under the
/// effect's controller's control (CR 400.7 — each is a new object).
///
/// Used by Sorin, Lord of Innistrad's -6: "Destroy up to three target creatures and/or
/// other planeswalkers. Return each card put into a graveyard this way to the battlefield
/// under your control."
///
/// Tokens that are destroyed do NOT return (they cease to exist as an SBA before the
/// reanimation step — CR 704.5d). Permanents redirected away from the graveyard by a
/// replacement effect (Rest in Peace, commander zone-change SBA, Kalitas) are likewise
/// not reanimated — only cards that actually land in a graveyard are returned.
DestroyAndReanimate {
    /// The permanents to destroy (typically `EffectTarget::DeclaredTarget`-style or an
    /// `AllPermanentsMatching`). Resolved via the standard target-list resolver.
    target: EffectTarget,
    /// CR 701.19c: if true, regeneration shields are bypassed during destruction.
    #[serde(default)]
    cant_be_regenerated: bool,
},
```

**Change 2b — execution logic**

**File**: `crates/engine/src/effects/mod.rs`
**Action**: add a new match arm. Place it immediately after the `Effect::DestroyAll { .. }`
arm (which ends around line 905). The arm reuses the existing `DestroyPermanent` destruction
logic and the existing `ReturnAllFromGraveyardToBattlefield` ETB-chain logic.

Implementation outline (the runner writes the full arm — this is the spec):
```rust
Effect::DestroyAndReanimate { target, cant_be_regenerated } => {
    // Phase 1 — destroy. Resolve targets, run the SAME destruction pipeline as
    // Effect::DestroyPermanent (indestructible check, regeneration shield check unless
    // cant_be_regenerated, umbra armor check, zone-change replacement check). For each
    // permanent that actually lands in a Graveyard zone, record (new_grave_id, owner).
    //
    // Collect `reanimate_ids: Vec<ObjectId>` = the `new_id` values from
    // ZoneChangeAction::Proceed branches AND from ZoneChangeAction::Redirect branches
    // where `dest` is ZoneId::Graveyard(_). Do NOT record Redirect-to-Exile,
    // Redirect-to-Command, or ChoiceRequired outcomes (those cards did not reach a
    // graveyard). Emit CreatureDied / PermanentDestroyed events exactly as
    // DestroyPermanent does (with pre_death_counters / pre_death_power LKI capture).
    //
    // Phase 2 — reanimate. For each id in reanimate_ids (sort ascending for determinism),
    // if the object is still in a Graveyard zone (it may have been moved by an SBA, e.g.
    // a token ceasing to exist, or a commander zone-change SBA between phase 1 and 2 —
    // guard with a zone check), move it to the battlefield under ctx.controller's control
    // and run the full ETB chain, exactly as Effect::ReturnAllFromGraveyardToBattlefield
    // does: move_object_to_zone -> set obj.controller = ctx.controller -> apply_self_etb_
    // from_definition -> apply_etb_replacements -> register_permanent_replacement_abilities
    // -> register_static_continuous_effects -> emit PermanentEnteredBattlefield.
}
```

**CRITICAL ordering note (CR 704.5 / SBA)**: between phase 1 and phase 2 no SBA check runs
inside the effect — effects resolve fully before SBAs (CR 704.3). Tokens destroyed in phase 1
are moved to the graveyard as objects; they will only cease to exist when SBAs next run
*after* the whole effect resolves. So a token *would* be momentarily in the graveyard during
phase 2. Per the Sorin ruling ("you can target a token... but it won't return"), the runner
MUST exclude tokens from phase-2 reanimation: skip any `reanimate_ids` entry whose object has
`is_token == true` OR `card_id == None`. (A reanimated token on the battlefield would also be
wrong — tokens can't be cast/reanimated.) Add an explicit `is_token` / `card_id.is_some()`
guard in phase 2.

**Recommended factoring**: to avoid copy-pasting ~90 lines of the `DestroyPermanent`
pipeline, extract the per-object destruction body (lines ~724-903 of effects/mod.rs, the
loop body inside `Effect::DestroyPermanent`) into a private helper
`fn destroy_one(state: &mut GameState, id: ObjectId, cant_be_regenerated: bool,
events: &mut Vec<GameEvent>) -> Option<ObjectId>` that returns `Some(new_grave_id)` when the
permanent landed in a graveyard, `None` otherwise. Then both `Effect::DestroyPermanent` and
`Effect::DestroyAndReanimate` call it. Similarly the phase-2 reanimate body is the loop body
of `ReturnAllFromGraveyardToBattlefield` (effects/mod.rs:5117-5167) — extract
`fn reanimate_one(state, old_id, controller, tapped, events)`. **If the runner judges the
extraction too invasive for a LOW batch, inline-duplicate is acceptable** — but the helper is
preferred and reduces the regression surface.

### Hash wiring

**File**: `crates/engine/src/state/hash.rs`
**Action**: add a `HashInto` arm for `Effect::DestroyAndReanimate` inside `impl HashInto for
Effect` (the match ends at line 5872). Highest discriminant currently in use is **84**
(`AddManaOfChosenColor`). Use **85**:
```rust
// PB-LS6: DestroyAndReanimate (discriminant 85) — CR 701.7 + reanimation
Effect::DestroyAndReanimate { target, cant_be_regenerated } => {
    85u8.hash_into(hasher);
    target.hash_into(hasher);
    cant_be_regenerated.hash_into(hasher);
}
```
Bump `HASH_SCHEMA_VERSION` in `hash.rs` by 1 (current value: read the `pub const` at the top
of `hash.rs` — it is the M9.5/PB-chain value, last bumped to 24 per CLAUDE.md "HASH 23→24").
New value: **25**. Update the parity test assertion (`assert_eq!(HASH_SCHEMA_VERSION, 25)`)
and the `hash.rs` module comment. Document the bump in the commit message.

### Card definition fix

**File**: `crates/engine/src/cards/defs/sorin_lord_of_innistrad.rs`
**Current state**: -6 ability (lines 57-71) is a `Sequence([DestroyPermanent x3])` with a
`TODO(PB-T-L02)` comment — the destroy half works, the reanimate rider is missing.
**Fix**: replace the `Sequence` effect with the new single variant. The `targets:` field
stays exactly as-is (`UpToN { count: 3, inner: TargetPermanentWithFilter
{ has_card_types: [Creature, Planeswalker] } }`).
```rust
// −6: Destroy up to three target creatures and/or other planeswalkers. Return each card
//     put into a graveyard this way to the battlefield under your control. (CR 601.2c)
AbilityDefinition::LoyaltyAbility {
    cost: LoyaltyCost::Minus(6),
    effect: Effect::DestroyAndReanimate {
        target: EffectTarget::DeclaredTarget { index: 0 },
        cant_be_regenerated: false,
    },
    targets: vec![TargetRequirement::UpToN {
        count: 3,
        inner: Box::new(TargetRequirement::TargetPermanentWithFilter(TargetFilter {
            has_card_types: vec![CardType::Creature, CardType::Planeswalker],
            ..Default::default()
        })),
    }],
},
```
Delete the `TODO(PB-T-L02)` comment. Update the descriptive comment to note the rider is now
implemented.

**`EffectTarget::DeclaredTarget { index: 0 }` semantics**: the `target` resolver
(`resolve_effect_target_list`) for an `UpToN` requirement at index 0 must resolve to the
full set of declared targets in that UpToN slot. Verify this in the runner — if
`DeclaredTarget { index: 0 }` only resolves to the *first* declared target, the destroy must
instead iterate indices 0..3. Check `resolve_effect_target_list` behavior against the
existing Tamiyo -2 / Elder Deep-Fiend pattern (those use explicit `DeclaredTarget { index: 0
/ 1 / 2 / 3 }` per-slot, which is the safe pattern). **If `DeclaredTarget` is strictly
single-target**, give `DestroyAndReanimate` a `targets: Vec<EffectTarget>` field instead and
list `[DeclaredTarget{0}, DeclaredTarget{1}, DeclaredTarget{2}]`, OR keep `target:
EffectTarget` but require the card def to wrap in `Effect::ForEach`. RUNNER MUST VERIFY this
before finalizing the variant shape — the Sorin -6 destroys *all* up-to-3 targets, not one.
Recommended: model `target` as a single `EffectTarget` and have the execution arm use
`resolve_effect_target_list` which, for the `UpToN` declared-targets case, should yield all
declared objects. The existing `DestroyAll` uses a filter; `DestroyPermanent` uses a single
`EffectTarget` + `resolve_effect_target_list` which DOES return a list. Confirm `DeclaredTarget`
under an `UpToN` requirement resolves to the whole slot — grep `resolve_effect_target_list`.

### Risks & edge cases (L02)

- **Tokens**: must be excluded from reanimation (guard on `is_token` / `card_id`). Covered above.
- **Replacement redirects**: Rest in Peace (destroy→exile), Kalitas, commander zone-change
  SBA all mean the card never reaches a graveyard — those must not be reanimated. The
  phase-1 record only captures graveyard outcomes. Covered above.
- **Sorin targeting itself** (ruling 2): if Sorin is somehow a creature and targets itself,
  it is destroyed and reanimated under controller's control — handled naturally (it's a
  nontoken card; it lands in graveyard, gets returned). Fine.
- **Undying/Persist on a destroyed creature**: the creature is moved to the graveyard
  (CreatureDied event emitted in phase 1), undying/persist triggers are queued by
  `check_triggers`, but phase 2 reanimates the card BEFORE those triggers resolve. When the
  undying trigger later resolves it finds the card already on the battlefield (new ObjectId)
  — the trigger's `MoveZone` from graveyard finds nothing and does nothing. This matches the
  Sorin ruling. No special handling needed; just confirm in a test.
- **`enrich_spec_from_def` / characteristics**: reanimated cards re-enter via
  `move_object_to_zone` + the ETB chain — `register_static_continuous_effects` repopulates
  characteristics from the card registry. Same path as `ReturnAllFromGraveyardToBattlefield`,
  proven for Living Death etc.

---

## Issue 3 — PB-T-L03: Tamiyo -2 freeze rider ("doesn't untap next untap step")

### Oracle text (confirmed via MCP)

Tamiyo, Field Researcher −2: "Tap up to two target nonland permanents. **They don't untap
during their controller's next untap step.**"

### Design decision: per-object counter, NOT a continuous effect

"Doesn't untap during the next untap step" is an untap-step exception (CR 502.3 — "effects
can keep one or more of a player's permanents from untapping"), not a continuous
characteristic-modifying effect. Modeling it as a `ContinuousEffect` /
`EffectDuration::UntilControllersNextUntapStep` would be wrong: it doesn't modify any
characteristic, and the duration is awkward (it must survive until the controller's untap
step then expire, but the permanent might change controllers in between — the oracle says
"their controller's next untap step", evaluated at each untap step).

**Decision**: add a per-object `skip_untap_steps: u32` field on `GameObject`, following the
established **EOC-flag pattern** (`decayed_sacrifice_at_eoc`, `myriad_exile_at_eoc`). It is a
*count* (not a bool) so that two separate freeze effects on the same permanent stack
correctly (CR has no special rule against it; e.g. two Tamiyos / a Tamiyo + Hands of Binding
on the same permanent should skip two untap steps). The untap step decrements it instead of
untapping.

This means **no `EffectDuration` change at all** — the Tamiyo card-def comment's reference to
`EffectDuration::UntilControllersNextUntapStep` and `LayerModification::PreventUntap` is
discarded. Fewer enum touches, no `EffectDuration` hash arm change.

### Engine Changes

**Change 3a — new `GameObject` field**

**File**: `crates/engine/src/state/game_object.rs`
**Action**: add a field to `struct GameObject` (after `ring_block_sacrifice_at_eoc`, around
line 807 — keep it grouped with the other per-turn/per-combat tracking flags):
```rust
/// CR 502.3: Number of upcoming untap steps during which this permanent must skip
/// untapping. Decremented (not zeroed) once per the controller's untap step in
/// `untap_active_player_permanents`; the permanent does not untap while this is > 0.
///
/// Set by `Effect::PreventNextUntap` (Tamiyo, Field Researcher -2; Hands of Binding).
/// A count rather than a bool so multiple freeze effects on the same permanent stack.
/// Reset to 0 on zone changes (CR 400.7) — a new object has no memory of the freeze.
#[serde(default)]
pub skip_untap_steps: u32,
```

**Change 3b — initialize the field at every `GameObject` struct-literal site**

`#[serde(default)]` covers deserialization, but explicit struct literals (no
`..Default::default()`) need the field added. The 12 explicit sites are exactly the ones that
already list `decayed_sacrifice_at_eoc: false` — grep confirmed:
- `crates/engine/src/state/mod.rs` — lines ~452, ~627, ~745, ~923
- `crates/engine/src/rules/resolution.rs` — lines ~4461, ~4661, ~5374, ~6034, ~6245, ~6471
- `crates/engine/src/state/builder.rs` — line ~1041
- `crates/engine/src/effects/mod.rs` — lines ~3436, ~4312, ~4478, ~6655

At each, add `skip_untap_steps: 0,` adjacent to the existing `decayed_sacrifice_at_eoc:
false,`. (Line numbers are approximate — the runner greps `decayed_sacrifice_at_eoc` and adds
the new field beside every hit that is a struct-literal field, NOT beside the hash.rs / type
doc / read-site hits.)

**Change 3c — reset the field on zone change**

`move_object_to_zone` in `crates/engine/src/state/mod.rs` already resets the EOC flags when
constructing the destination object (the struct-literal sites at mod.rs ~627 and ~745 are the
zone-change construction paths). Adding `skip_untap_steps: 0` there (Change 3b) is the reset
— no separate code needed. Confirm: the comment block above `decayed_sacrifice_at_eoc: false`
at mod.rs ~451/~922 says "CR 400.7: ... not preserved across zone changes" — add an analogous
one-line comment for `skip_untap_steps`.

**Change 3d — untap step honors the counter**

**File**: `crates/engine/src/rules/turn_actions.rs`
**Function**: `untap_active_player_permanents` (lines 1042-1227).
**Action**: modify the untap loop at lines 1204-1218. Currently:
```rust
// CR 502.2: Untap tapped permanents.
if obj.status.tapped {
    obj.status.tapped = false;
    untapped.push(*id);
}
```
Replace with:
```rust
// CR 502.3: a permanent with a pending skip-untap count does not untap this
// step; instead the count is decremented by one (PB-T-L03, Tamiyo -2 / Hands of
// Binding). Once the count reaches 0 it untaps normally on a later untap step.
if obj.skip_untap_steps > 0 {
    obj.skip_untap_steps -= 1;
} else if obj.status.tapped {
    obj.status.tapped = false;
    untapped.push(*id);
}
```
**Important**: the decrement happens for the permanent whose *controller* is the active
player (the loop at 1204 iterates `ids_on_battlefield` = battlefield permanents controlled
by `active`). This correctly implements "their controller's next untap step" — the counter
is consumed only on the untap step of whoever currently controls the permanent. If the
permanent changes controllers before that untap step, it skips the *new* controller's untap
step — matches the oracle ("their controller's next untap step", re-evaluated). Even an
*untapped* frozen permanent decrements the counter (the `else if obj.status.tapped` ordering
ensures the decrement always fires when count > 0, regardless of tapped state) — correct, the
"next untap step" is consumed whether or not the permanent happened to be tapped.

**Change 3e — new `Effect` variant `PreventNextUntap`**

**File**: `crates/engine/src/cards/card_definition.rs`
**Action**: add near `Effect::TapPermanent` / `Effect::UntapPermanent` (around line 1314):
```rust
/// CR 502.3: The target permanent doesn't untap during its controller's next untap
/// step. Increments the target's `skip_untap_steps` counter by 1; the counter is
/// decremented at the controller's untap step (see `untap_active_player_permanents`).
///
/// Used by Tamiyo, Field Researcher -2 and Hands of Binding. Stacks: applying this
/// twice makes the permanent skip two untap steps.
PreventNextUntap { target: EffectTarget },
```

**Change 3f — execute `PreventNextUntap`**

**File**: `crates/engine/src/effects/mod.rs`
**Action**: add a match arm immediately after `Effect::UntapPermanent { .. }` (ends ~line
1593):
```rust
Effect::PreventNextUntap { target } => {
    let targets = resolve_effect_target_list(state, target, ctx);
    for resolved in targets {
        if let ResolvedTarget::Object(id) = resolved {
            if let Some(obj) = state.objects.get_mut(&id) {
                obj.skip_untap_steps = obj.skip_untap_steps.saturating_add(1);
            }
        }
    }
}
```
No game event is emitted — "doesn't untap" produces no observable event of its own (it is
the *absence* of a future `PermanentsUntapped`). This matches how no event is emitted for
the freeze itself; the only observable is the missing untap. (If a future card needs a
"permanent was frozen" trigger, add an event then — out of scope now.)

### Hash wiring (L03)

**File**: `crates/engine/src/state/hash.rs`

1. **`GameObject` field** — add `self.skip_untap_steps.hash_into(hasher);` in
   `impl HashInto for GameObject`, beside the existing
   `self.decayed_sacrifice_at_eoc.hash_into(hasher);` / `ring_block_sacrifice_at_eoc` lines
   (~1170-1172). `u32` has a `HashInto` impl already.
2. **`Effect::PreventNextUntap` arm** — add to `impl HashInto for Effect` (before the
   closing `}` at line 5872). Discriminant: **86** (85 taken by `DestroyAndReanimate`):
   ```rust
   // PB-LS6: PreventNextUntap (discriminant 86) — CR 502.3
   Effect::PreventNextUntap { target } => {
       86u8.hash_into(hasher);
       target.hash_into(hasher);
   }
   ```
3. The `HASH_SCHEMA_VERSION` bump to **25** (from Change 2b) covers both new Effect variants
   and the new `GameObject` field — a single bump for the whole batch. Do NOT bump twice.

### Card definition fixes (L03)

**File 1**: `crates/engine/src/cards/defs/tamiyo_field_researcher.rs`
**Current**: -2 (lines 34-47) is `Sequence([TapPermanent{0}, TapPermanent{1}])` with a
`TODO(PB-T-L03)` comment; tap half works, freeze rider missing.
**Fix**: extend the `Sequence` to tap then freeze each target:
```rust
// −2: Tap up to two target nonland permanents. (CR 601.2c / 115.1b)
// They don't untap during their controller's next untap step. (CR 502.3)
AbilityDefinition::LoyaltyAbility {
    cost: LoyaltyCost::Minus(2),
    effect: Effect::Sequence(vec![
        Effect::TapPermanent { target: EffectTarget::DeclaredTarget { index: 0 } },
        Effect::TapPermanent { target: EffectTarget::DeclaredTarget { index: 1 } },
        Effect::PreventNextUntap { target: EffectTarget::DeclaredTarget { index: 0 } },
        Effect::PreventNextUntap { target: EffectTarget::DeclaredTarget { index: 1 } },
    ]),
    targets: vec![TargetRequirement::UpToN {
        count: 2,
        inner: Box::new(TargetRequirement::TargetPermanentWithFilter(TargetFilter {
            non_land: true,
            ..Default::default()
        })),
    }],
},
```
Delete the `TODO(PB-T-L03)` comment and the incorrect "CR 613.6" reference; cite CR 502.3.
Note: a `DeclaredTarget` index that has no declared target (player chose only 1 of 2)
resolves to an empty list — both `TapPermanent` and `PreventNextUntap` no-op on the missing
index. Correct for "up to two".

**File 2 (forced add from TODO sweep)**: `crates/engine/src/cards/defs/hands_of_binding.rs`
**Current**: Spell effect (lines 19-29) is `TapPermanent{0}` only, with TODO comments at
lines 6-8 and 21 naming the missing primitive.
**Fix**: change the spell effect to a `Sequence`:
```rust
AbilityDefinition::Spell {
    // CR 702.27a: Tap target creature an opponent controls.
    // CR 502.3: it doesn't untap during its controller's next untap step.
    effect: Effect::Sequence(vec![
        Effect::TapPermanent { target: EffectTarget::DeclaredTarget { index: 0 } },
        Effect::PreventNextUntap { target: EffectTarget::DeclaredTarget { index: 0 } },
    ]),
    targets: vec![TargetRequirement::TargetCreature],
    modes: None,
    cant_be_countered: false,
},
```
Delete the file-header TODO (lines 6-8) and the inline TODO (line 21). Note: Hands of
Binding's `TargetRequirement::TargetCreature` does not restrict to "an opponent controls" —
that is a pre-existing card-def gap unrelated to this batch; leave it (do not expand scope).
The freeze-rider TODO is resolved; that is the L03 deliverable.

### Risks & edge cases (L03)

- **Permanent already untapped when frozen**: the counter still decrements at the next untap
  step (the `else if obj.status.tapped` guard ensures decrement-always-when-count>0). If the
  permanent is untapped and frozen, then tapped by something else before its untap step, it
  correctly stays tapped through that untap step. Correct.
- **Controller change**: counter is consumed on the *current* controller's untap step. CR /
  oracle says "their controller's next untap step" — re-evaluated, so this is correct.
- **Phasing**: a phased-out permanent is excluded from `ids_on_battlefield` (the loop filters
  `!obj.status.phased_out`), so its `skip_untap_steps` is NOT decremented while phased out.
  When it phases back in and reaches an untap step, the counter is still pending. This is a
  benign edge case (phasing + freeze is extremely rare) and arguably correct (the permanent
  didn't have an untap step while phased out). Note it; no special handling.
- **Hash determinism**: `skip_untap_steps` is now part of the public state hash — distributed
  peers agree. Required (it affects future untap behavior).

---

## Exhaustive-Match / Wiring Checklist

| File | Match / site | Action |
|------|--------------|--------|
| `crates/engine/src/cards/card_definition.rs` | `enum Effect` | Add `DestroyAndReanimate` (~L1294) + `PreventNextUntap` (~L1314) variants |
| `crates/engine/src/state/game_object.rs` | `struct GameObject` | Add `skip_untap_steps: u32` field (~L807) |
| `crates/engine/src/effects/mod.rs` | `match effect` in effect executor | Add `DestroyAndReanimate` arm (after `DestroyAll`, ~L905) + `PreventNextUntap` arm (after `UntapPermanent`, ~L1593) |
| `crates/engine/src/state/hash.rs` | `impl HashInto for Effect` | Add discriminant 85 (`DestroyAndReanimate`) + 86 (`PreventNextUntap`) before `}` at ~L5872 |
| `crates/engine/src/state/hash.rs` | `impl HashInto for GameObject` | Add `self.skip_untap_steps.hash_into(hasher);` near ~L1170 |
| `crates/engine/src/state/hash.rs` | `HASH_SCHEMA_VERSION` const + parity test + module comment | Bump 24 → 25 (single bump for whole batch) |
| `crates/engine/src/state/mod.rs` | 4 `GameObject` struct literals (~L452, ~627, ~745, ~923) | Add `skip_untap_steps: 0,` |
| `crates/engine/src/rules/resolution.rs` | 6 `GameObject` struct literals (~L4461, 4661, 5374, 6034, 6245, 6471) | Add `skip_untap_steps: 0,` |
| `crates/engine/src/state/builder.rs` | 1 `GameObject` struct literal (~L1041) | Add `skip_untap_steps: 0,` |
| `crates/engine/src/effects/mod.rs` | 3 `GameObject` struct literals (~L3436, 4312, 4478, 6655 — 4 hits) | Add `skip_untap_steps: 0,` |
| `crates/engine/src/rules/turn_actions.rs` | `untap_active_player_permanents` untap loop (~L1213) | Decrement `skip_untap_steps` instead of untapping when > 0 |
| `crates/engine/src/rules/engine.rs` | `handle_activate_loyalty_ability` (~L2292, ~L2315) | Bind `ability_targets`, add `validate_targets_with_source` call |

**Authoritative way to find all struct-literal sites**: `grep -rn "decayed_sacrifice_at_eoc"
crates/engine/src/` — every hit that is a struct field literal (`decayed_sacrifice_at_eoc:
false,`) needs `skip_untap_steps: 0,` beside it. Hits in `hash.rs` (a `.hash_into` call),
`types.rs` (doc comment), and `turn_actions.rs`/`combat.rs` (field reads/writes) are NOT
struct literals — do not touch those for field-init purposes.

**Tools crates**: confirmed via grep — `tools/tui/` and `tools/replay-viewer/` do NOT match
on `Effect`, `EffectDuration`, or `LayerModification` (they match on `StackObjectKind` and
`KeywordAbility` only, neither of which changes here). **No tools-crate edits required.**
Still run `cargo build --workspace` after the implement phase to be certain.

**`replay_harness.rs`**: has an `if let Effect::Sequence(effects) = effect` at L3187 (a
specific destructure, not an exhaustive match) — adding new `Effect` variants does not break
it. No harness edit needed unless a test script needs a new action type — the unit tests
below use the engine API directly, so no harness action is required.

---

## Unit Tests

**File**: `crates/engine/tests/loyalty_target_validation.rs` (new file) — for L01.
Pattern: follow existing loyalty/planeswalker tests; grep `tests/` for `LoyaltyAbility` or
`ActivateLoyaltyAbility` usages and mirror the builder setup.
- `test_l01_loyalty_ability_rejects_illegal_target_type` — activate Sorin -6 targeting a
  Land object; assert `process_command` returns `Err(InvalidTarget)` and Sorin's loyalty
  counter is unchanged (still 3, cost not paid). CR 601.2c / 606.4.
- `test_l01_loyalty_ability_accepts_legal_target` — Sorin -6 targeting a real creature;
  assert `Ok`, loyalty went 6→0, ability on stack.
- `test_l01_loyalty_ability_zero_targets_legal` — Sorin -6 (UpToN count 3) with 0 declared
  targets; assert `Ok` (min targets = 0). CR 601.2c.
- `test_l01_loyalty_cost_not_paid_on_rejected_activation` — explicit assertion that an
  `InvalidTarget` rejection leaves `CounterType::Loyalty` and
  `loyalty_ability_activated_this_turn` untouched.
- `test_l01_no_target_ability_unaffected` — Sorin +1 (targets: vec![]) still activates fine.

**File**: `crates/engine/tests/destroy_and_reanimate.rs` (new file) — for L02.
Pattern: follow `tests/` reanimation tests (grep `ReturnAllFromGraveyardToBattlefield` /
`Living Death` test files) and the `DestroyPermanent` tests.
- `test_l02_destroy_and_reanimate_basic` — controller P1, an opponent P2 creature on the
  battlefield. Resolve Sorin -6 targeting it. Assert: the creature card is now on the
  battlefield, `controller == P1`, `owner == P2`, and it is a NEW ObjectId. CR 701.7 + 400.7.
- `test_l02_reanimate_under_your_control` — destroy three creatures owned by 3 different
  players; assert all three return controlled by the activating player.
- `test_l02_token_destroyed_not_reanimated` — target a token creature; assert it is
  destroyed (CreatureDied) but does NOT return (no battlefield object with that name after
  resolution + SBA). Sorin ruling 2011-01-22.
- `test_l02_rest_in_peace_redirect_not_reanimated` — Rest in Peace in play (destroy→exile
  replacement); Sorin -6 on a creature; assert the creature is exiled and NOT reanimated.
  CR 614.
- `test_l02_indestructible_target_survives` — target an indestructible creature; assert it
  is neither destroyed nor reanimated (stays on battlefield, same ObjectId).
- `test_l02_destroy_and_reanimate_runs_etb` — destroyed creature has an ETB trigger; assert
  the trigger fires when it is reanimated (PermanentEnteredBattlefield emitted, trigger
  queued). CR 603.6a.

**File**: `crates/engine/tests/prevent_next_untap.rs` (new file) — for L03.
Pattern: follow `tests/` turn-structure / untap-step tests; use a 2+ player builder so the
full turn cycle reaches an untap step (1-player games end immediately — see gotchas).
- `test_l03_frozen_permanent_skips_one_untap_step` — tap a permanent, apply
  `PreventNextUntap`, advance to its controller's next untap step; assert it stays tapped
  and `skip_untap_steps` is now 0; advance one more full turn cycle; assert it untaps. CR 502.3.
- `test_l03_freeze_stacks` — apply `PreventNextUntap` twice; assert it skips TWO untap steps
  before untapping (`skip_untap_steps` 2→1→0).
- `test_l03_freeze_counter_reset_on_zone_change` — freeze a permanent, then move it to hand
  and back to battlefield; assert the new object has `skip_untap_steps == 0`. CR 400.7.
- `test_l03_tamiyo_minus2_freezes_targets` — full integration: activate Tamiyo -2 on two
  nonland permanents; assert both are tapped AND both have `skip_untap_steps == 1`; advance
  to their untap step; assert they stay tapped.
- `test_l03_hands_of_binding_freezes_target` — cast Hands of Binding on an opponent's
  creature; assert tapped + `skip_untap_steps == 1`; advance to the opponent's untap step;
  assert it stays tapped.
- `test_l03_untapped_frozen_permanent_consumes_count` — apply `PreventNextUntap` to an
  *untapped* permanent; advance to its untap step; assert `skip_untap_steps` decremented to 0
  (the "next untap step" was consumed even though nothing untapped).

---

## Verification Checklist

- [ ] `Effect::DestroyAndReanimate` + `Effect::PreventNextUntap` variants compile
- [ ] `GameObject.skip_untap_steps` field added; all 12 explicit struct-literal sites updated
- [ ] `handle_activate_loyalty_ability` binds `ability_targets` and validates
- [ ] `untap_active_player_permanents` decrements `skip_untap_steps`
- [ ] `HASH_SCHEMA_VERSION` bumped 24 → 25; parity test assertion updated; module comment updated
- [ ] `HashInto for Effect` arms for discriminants 85 & 86; `HashInto for GameObject` hashes new field
- [ ] Sorin -6, Tamiyo -2, Hands of Binding card defs updated; all `TODO(PB-T-L0x)` comments deleted
- [ ] `cargo check -p mtg-engine` clean
- [ ] `cargo build --workspace` clean (confirms no tools-crate breakage)
- [ ] `cargo test --all` passes (new tests + no regressions in 2819-test baseline)
- [ ] `cargo clippy -- -D warnings` clean (watch `large_enum_variant` if a new variant is big —
      `DestroyAndReanimate` holds only `EffectTarget` + `bool`, `PreventNextUntap` only
      `EffectTarget`; both small, no boxing needed)
- [ ] No remaining `TODO` in `sorin_lord_of_innistrad.rs`, `tamiyo_field_researcher.rs`,
      `hands_of_binding.rs` referencing PB-T-L02 / PB-T-L03 / the freeze or reanimate rider

---

## Cross-Cutting Risks

- **`DeclaredTarget` under `UpToN` resolution** (L02) — the single biggest open question:
  the runner MUST verify `resolve_effect_target_list(state, &EffectTarget::DeclaredTarget
  { index: 0 }, ctx)` returns ALL declared targets in the up-to-3 slot, not just the first.
  If it returns only one, switch `DestroyAndReanimate.target` to iterate or change Sorin's
  card def to wrap per-index. Grep `resolve_effect_target_list` + check how Elder Deep-Fiend
  (uses explicit `index: 0/1/2/3`) vs. a filter-based destroy resolves. This determines the
  final variant shape — resolve before writing the execution arm.
- **Helper extraction vs. inline-duplicate** (L02) — extracting `destroy_one` /
  `reanimate_one` touches `DestroyPermanent` and `ReturnAllFromGraveyardToBattlefield`. If
  the runner extracts, run the existing DestroyPermanent / Living Death / Eerie Ultimatum
  tests to confirm no behavior change. If inline-duplicate, the duplication is ~90 lines —
  acceptable but flag it for the reviewer.
- **Hash bump discipline** — one bump (24→25) for the whole batch. Two new Effect variants +
  one new GameObject field all ride the single bump. Document in the commit message and the
  `hash.rs` module comment per `memory/conventions.md` "Hash sentinel convention".
- **Loyalty validation ordering** (L01) — validation MUST run before the loyalty-cost
  mutation at engine.rs:2317. A test explicitly asserts loyalty is untouched on rejection.
- **No `EffectDuration` change** — confirmed: L03 uses a per-object counter, not a duration.
  The Tamiyo card-def comment's reference to `EffectDuration::UntilControllersNextUntapStep`
  is discarded; do not add that variant.
