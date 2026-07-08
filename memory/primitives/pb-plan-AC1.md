# Primitive Batch Plan: PB-AC1 — Counter / Untap / Once-per-turn

**Generated**: 2026-07-07
**Primitive**: Five DSL capabilities — `Effect::UntapAll { filter }`;
`TriggerCondition::WheneverPermanentUntaps`; `TriggerCondition::WhenCounterPlaced`;
a generic `once_per_turn` limiter on triggered abilities; and a
`KeywordAbility::DoesNotUntap` static ("doesn't untap during its controller's untap step").
**CR Rules (VERIFIED via MCP)**: 701.26 (Tap and Untap keyword action), 502.3 (untap
step: active player untaps their permanents; "effects can keep one or more of a player's
permanents from untapping"), 502.4 (no priority in untap step; triggers held to upkeep),
603.2 / 603.2c / 603.2e / 603.2h (triggered abilities; "becomes" events; once-per-turn),
603.3 / 603.3b (putting triggers on the stack, APNAP), 122 / 122.6 / 122.7 (counters put on).
**Cards affected**: ~15 touched (8 fully-clean, ~7 partial). Discounted from the plan's
advisory ~22 per `feedback_pb_yield_calibration.md` (~2.5x).
**Dependencies**: none (all prerequisite primitives — `Effect::UntapPermanent`,
`Effect::PreventNextUntap`, `Effect::MillCards`, `WheneverCreatureDies`, `Forecast`,
`Evolve`, `Adapt`, `TargetFilter` — already exist).
**Deferred items from prior PBs**: none applicable (per `primitive-wip.md`).

> **CR-ref correction (advisory refs were wrong).** The task brief and
> `campaign-plan-2026-05-16.md §2` cite **701.20 / 701.21** for untap. **701.20 is
> "Reveal"** (verified via MCP), not untap. The untap keyword action is **701.26**;
> the untap **step** is **502.3** (with 502.4 for the no-priority/held-triggers rule).
> Use 701.26 / 502.3 / 502.4 in all comments and test citations. Do not cite 701.20/701.21.

---

## Primitive Specification

Five additions, all low-novelty (untap and counters are well-trodden in the engine):

1. **`Effect::UntapAll { filter: TargetFilter }`** — one-shot effect that untaps every
   battlefield permanent matching `filter` (CR 701.26b: only tapped permanents untap).
   `filter.controller` (`TargetController::You / Opponent / Any`) supplies the "you
   control" scoping — no separate `controller_only` bool. Mirrors `Effect::ExileAll`.

2. **`TriggerCondition::WheneverPermanentUntaps { filter: Option<TargetFilter> }`** — a
   GLOBAL trigger that fires when any permanent matching `filter` becomes untapped
   (`None` = any permanent, any controller). CR 603.2e: fires only on the untap *event*,
   never because a permanent enters untapped. Canonical card: Mesmeric Orb.

3. **`TriggerCondition::WhenCounterPlaced { counter: Option<CounterType>, filter:
   Option<TargetFilter>, on_self: bool }`** — fires when one or more counters (of the
   given kind; `None` = any) are put on a permanent. `on_self: true` = "on this
   creature/permanent"; `on_self: false` + `filter` = "on a [creature you control]".
   CR 122.6 / 122.7. Canonical cards: Fathom Mage, Dusk Legion Duelist, Sharktocrab,
   Simic Ascendancy.

4. **Generic `once_per_turn` limiter on triggered abilities** — a `bool` field on
   `AbilityDefinition::Triggered` (and the runtime `TriggeredAbilityDef`). When set, the
   ability is put on the stack at most once per turn regardless of how many times its
   condition is met (CR 603.2h anchor; card text "This ability triggers only once each
   turn"). Canonical card: Morbid Opportunist.

5. **`KeywordAbility::DoesNotUntap`** — an engine-internal pseudo-keyword (NOT a CR
   keyword) representing the static "This permanent doesn't untap during its controller's
   untap step" (CR 502.3). Modeled as a keyword so it lives in layer-resolved
   `Characteristics.keywords` and is therefore removed by Humility / Dress Down
   (`LayerModification::RemoveAllAbilities` clears keywords). This EXACTLY mirrors the
   existing `KeywordAbility::CantBlock` pseudo-keyword (PB-36, disc 160). Canonical cards:
   Goblin Sharpshooter, Mana Vault.

---

## CR Rule Text (verified via MCP)

**701.26 Tap and Untap**
- 701.26a To tap a permanent, turn it sideways from an upright position. Only untapped
  permanents can be tapped.
- 701.26b To untap a permanent, rotate it back to the upright position from a sideways
  position. **Only tapped permanents can be untapped.**

**502.3** Third, the active player determines which permanents they control will untap.
Then they untap them all simultaneously. This turn-based action doesn't use the stack.
Normally, all of a player's permanents untap, but **effects can keep one or more of a
player's permanents from untapping.**

**502.4** No player receives priority during the untap step... Any ability that triggers
during this step **will be held until the next time a player would receive priority,
which is usually during the upkeep step.** (Confirms the WheneverPermanentUntaps triggers
from the untap step correctly queue and fire at upkeep — the engine already holds
`pending_triggers` until priority.)

**603.2c** An ability triggers only once each time its trigger event occurs. However, it
can trigger repeatedly if one event contains multiple occurrences.

**603.2e** Some trigger events use the word "becomes"... An ability that triggers when a
permanent "becomes tapped" or "becomes untapped" **doesn't trigger if the permanent enters
the battlefield in that state.**

**603.2h** A triggered ability may have an instruction followed by "Do this only once each
turn." This ability triggers only if its source's controller has not yet taken the
indicated action that turn. *(Closest CR anchor for the once-per-turn limiter. Morbid
Opportunist's exact text is "This ability triggers only once each turn"; the engine models
both as "put on the stack at most once per turn." Note the fine CR distinction between
"triggers only once" vs "do this only once" is not separately modeled — the observable
result is identical: one instance per turn.)*

**122.6** Some spells and abilities refer to counters being put on an object. This refers
to putting counters on that object while it's on the battlefield **and also to an object
that's given counters as it enters the battlefield.**

**122.7** An ability that triggers "When/Whenever the Nth [kind] counter" is put on an
object triggers when one or more counters of the appropriate kind are put on the object
such that the object had fewer than N... *(Our roster is all "whenever one or more counters
are put on," i.e. N=1 — fires once per placement event where ≥1 counter of the kind lands.)*

**Morbid Opportunist ruling (2024-11-08)**: If Morbid Opportunist dies at the same time as
one or more other creatures, its ability still triggers. (Death-batch + once-per-turn; the
existing `WheneverCreatureDies { exclude_self:false }` already look-backs correctly.)

---

## Discriminant Chain (VERIFIED against current code — use these EXACT values)

The planner has historically mis-assigned these; these were read directly from `hash.rs`
on 2026-07-07. Verify once more before writing, then use verbatim.

| Enum | Current max | New variant(s) | New disc |
|------|-------------|----------------|----------|
| `Effect` (hash.rs:5181) | 86 (`PreventNextUntap`) | `UntapAll { filter }` | **87** |
| `TriggerCondition` (hash.rs:4925 end) | 41 (`WhenTappedForMana`) | `WheneverPermanentUntaps` | **42** |
| `TriggerCondition` | — | `WhenCounterPlaced` | **43** |
| `TriggerEvent` (runtime, hash.rs:2353) | 44 (`OpponentPlaysLand`) | `AnyPermanentUntaps` | **45** |
| `TriggerEvent` (runtime) | — | `CounterPlaced` | **46** |
| `KeywordAbility` (hash.rs:528) | 161 (`CantBeBlockedExceptBy`) | `DoesNotUntap` | **162** |
| `HASH_SCHEMA_VERSION` (hash.rs:202) | 27 | bump | **28** |

`once_per_turn: bool` on `AbilityDefinition::Triggered` and `TriggeredAbilityDef` is a
FIELD, not a variant — no discriminant, but bumps hash schema (field-shape change). The new
GameObject field `triggered_abilities_fired_this_turn` is also a field (hashed near
`skip_untap_steps` at hash.rs:1292).

---

## Engine Changes

### Change 1: `Effect::UntapAll` variant + execution

**File**: `crates/engine/src/cards/card_definition.rs` (~L1338, next to `UntapPermanent`
/ `PreventNextUntap`, near `ExileAll` at L1321).
**Action**: Add `UntapAll { filter: TargetFilter }` with a doc comment citing CR 701.26b /
502.3. Store an affected count in `ctx.last_effect_count` (mirror `ExileAll`).

**File**: `crates/engine/src/effects/mod.rs` (add arm after `Effect::UntapPermanent`
L1880, modeled on `Effect::ExileAll` L1418).
**Action**: Snapshot matching tapped permanents, untap each, emit an untap event per
permanent. Use the `ExileAll` filter-match idiom:
```rust
Effect::UntapAll { filter } => {
    let ids: Vec<ObjectId> = state.objects.iter()
        .filter(|(id, obj)| obj.zone == ZoneId::Battlefield && obj.is_phased_in() && obj.status.tapped && {
            let chars = calculate_characteristics(state, **id).unwrap_or_else(|| obj.characteristics.clone());
            matches_filter(&chars, filter) && check_chosen_subtype_filter(state, ctx, filter, &chars)
        } && match filter.controller {
            TargetController::Any => true,
            TargetController::You => obj.controller == ctx.controller,
            TargetController::Opponent => obj.controller != ctx.controller,
            TargetController::DamagedPlayer => obj.controller == ctx.damaged_player.unwrap_or(ctx.controller),
        })
        .map(|(&id, _)| id).collect();
    for id in &ids {
        if let Some(obj) = state.objects.get_mut(id) {
            obj.status.tapped = false;
            let player = obj.controller;
            events.push(GameEvent::PermanentUntapped { player, object_id: *id });
        }
    }
    ctx.last_effect_count = ids.len() as u32;
}
```
**CR**: 701.26b (only tapped permanents untap → the `obj.status.tapped` guard). Emitting
`PermanentUntapped` per permanent is what makes `WheneverPermanentUntaps` fire (Change 2).

**Note (helpers prelude)**: `UntapAll` uses `TargetFilter`, already exported. No new
`helpers.rs` export needed.

### Change 2: `TriggerCondition::WheneverPermanentUntaps` + dispatch

**File**: `crates/engine/src/cards/card_definition.rs` (TriggerCondition enum, near
`WhenSelfBecomesTapped` L2949).
**Action**: Add `WheneverPermanentUntaps { #[serde(default)] filter: Option<TargetFilter> }`.
Doc: CR 502.3 / 603.2e — GLOBAL trigger; `filter: None` = any permanent any controller.

**File**: `crates/engine/src/state/game_object.rs` (runtime `TriggerEvent` enum ~L467).
**Action**: Add `AnyPermanentUntaps`. Doc: fires on ALL battlefield permanents when a
permanent becomes untapped; the untapped permanent is carried via the `entering_object`
param (reused as "triggering object").

**File**: `crates/engine/src/testing/replay_harness.rs` — `enrich_spec_from_def` (L1840;
add a conversion loop next to the `WhenSelfBecomesTapped` loop at L2545).
**Action**: For each `AbilityDefinition::Triggered { trigger_condition:
WheneverPermanentUntaps { filter }, effect, once_per_turn, .. }`, push a `TriggeredAbilityDef`
with `trigger_on: TriggerEvent::AnyPermanentUntaps`, `triggering_creature_filter:
filter.clone()`, `once_per_turn`, all other filter fields `None`.

**File**: `crates/engine/src/rules/abilities.rs` — `check_triggers` (the big event match,
~L3488 has the `PermanentTapped` arm to mirror).
**Action**: Add arms for BOTH untap events:
```rust
GameEvent::PermanentUntapped { object_id, .. } => {
    // CR 502.3 / 603.2e: fire WheneverPermanentUntaps globally; triggering obj = the untapped permanent.
    collect_triggers_for_event(state, &mut triggers, TriggerEvent::AnyPermanentUntaps, None, Some(*object_id));
}
GameEvent::PermanentsUntapped { objects, .. } => {
    // CR 502.3/502.4: untap-step batch. One dispatch per untapped permanent; triggers are
    // held (pending_triggers) and go on the stack at upkeep.
    for id in objects {
        collect_triggers_for_event(state, &mut triggers, TriggerEvent::AnyPermanentUntaps, None, Some(*id));
    }
}
```
**File**: `crates/engine/src/rules/abilities.rs` — `collect_triggers_for_event` (L6027).
**Action**: In the loop, add a filter/skip block for `AnyPermanentUntaps` mirroring the
`AnyCreatureDies`/`AnyCreatureYouControlAttacks` handling: use `entering_object` as the
untapped permanent; if `triggering_creature_filter` is `Some`, evaluate `matches_filter`
against the untapped permanent's `calculate_characteristics`; also honor `filter.controller`
if the filter sets a controller scope. Ensure the pushed `PendingTrigger` sets
`entering_object_id = Some(untapped_id)` so the effect can resolve
`PlayerTarget::ControllerOf(EffectTarget::TriggeringCreature)` (Mesmeric Orb mills the
untapped permanent's controller). CR 603.2e is satisfied structurally — this path only runs
on real untap events, never on ETB.

### Change 3: `TriggerCondition::WhenCounterPlaced` + dispatch

**File**: `crates/engine/src/cards/card_definition.rs` (TriggerCondition enum).
**Action**: Add
```rust
WhenCounterPlaced {
    #[serde(default)] counter: Option<CounterType>,   // None = any kind
    #[serde(default)] filter: Option<TargetFilter>,   // for "on a [creature you control]"
    #[serde(default)] on_self: bool,                  // "on this creature/permanent"
},
```
Doc: CR 122.6 / 122.7 — fires when ≥1 counter of `counter` is put on a matching permanent.

**File**: `crates/engine/src/state/game_object.rs` (runtime `TriggerEvent`).
**Action**: Add `CounterPlaced`. The receiving permanent's ObjectId is carried via
`entering_object`.

**File**: `crates/engine/src/testing/replay_harness.rs` — `enrich_spec_from_def`.
**Action**: Conversion loop → `TriggeredAbilityDef { trigger_on: TriggerEvent::CounterPlaced,
triggering_creature_filter: filter.clone(), once_per_turn, .. }`. The `counter` and
`on_self` bits need to reach the dispatch: **carry them on the `TriggeredAbilityDef`**. Add
two fields to `TriggeredAbilityDef` (game_object.rs L585): `#[serde(default)] counter_filter:
Option<CounterType>` and `#[serde(default)] counter_on_self: bool`. (Both hashed; both
`#[serde(default)]` so existing serialized defs load.)

**File**: `crates/engine/src/rules/abilities.rs` — `check_triggers`.
**Action**: Add an arm:
```rust
GameEvent::CounterAdded { object_id, counter, count } if *count > 0 => {
    collect_triggers_for_event(state, &mut triggers, TriggerEvent::CounterPlaced, None, Some(*object_id));
    // pass the counter kind through EffectContext or a thread-local param — see below.
}
```
Because `collect_triggers_for_event` needs the counter kind to compare against
`counter_filter`, either (a) add a `counter_kind: Option<CounterType>` param to
`collect_triggers_for_event`, or (b) read the kind from the event inside the arm and filter
there. **Recommend (a)**: extend `collect_triggers_for_event` with an optional
`counter_context: Option<CounterType>` param (default `None` at all existing call sites).

**File**: `crates/engine/src/rules/abilities.rs` — `collect_triggers_for_event`.
**Action**: Add a `CounterPlaced` filter block:
- `on_self: true` → fire only if `entering_object == obj_id` (the trigger source received the
  counter).
- `on_self: false` → the receiving permanent (`entering_object`) must match
  `triggering_creature_filter` (subtype/type) AND, if the filter sets a controller,
  `receiving_obj.controller == obj.controller` for "you control".
- `counter_filter: Some(k)` → skip unless the placed `counter == k`.
- Set `entering_object_id = Some(receiving_id)` on the pushed trigger so
  `EffectTarget::TriggeringCreature` / `PlayerTarget::ControllerOf(...)` resolve, and so
  `EffectAmount::CounterCount { target: TriggeringCreature, .. }` works for cards that read
  the counters on the receiving permanent.

**Edge case (CR 122.6)**: enters-with-counters. If `CounterAdded` is NOT emitted for
counters an object enters with, "put on as it enters" won't trigger. This is a known
fidelity gap; note it in the plan and add a test asserting current behavior (do not expand
scope to fix ETB-with-counters emission in this PB).

### Change 4: `once_per_turn` limiter on triggered abilities

**File**: `crates/engine/src/cards/card_definition.rs` — `AbilityDefinition::Triggered`
(L247–269).
**Action**: Add `#[serde(default)] once_per_turn: bool` field. **CAUTION**: every existing
`AbilityDefinition::Triggered { .. }` construction that does NOT use `..` struct-rest and
lists fields explicitly must add `once_per_turn: false`. Most card defs use the field-listed
form (see goblin_sharpshooter L19). Grep `TriggerCondition::` across `cards/defs/` and add
`once_per_turn: false,` to every explicit `Triggered { .. }` literal, plus builder/harness
constructors. This is the largest mechanical surface — budget for it.

**File**: `crates/engine/src/state/game_object.rs` — `TriggeredAbilityDef` (L585).
**Action**: Add `#[serde(default)] once_per_turn: bool`. All `with_triggered_ability`
call sites in `enrich_spec_from_def` (replay_harness.rs) must set it (forward from the card
def where the trigger has `once_per_turn`, else `false`).

**File**: `crates/engine/src/state/game_object.rs` — `GameObject` struct (near
`skip_untap_steps` L1175).
**Action**: Add `#[serde(default)] triggered_abilities_fired_this_turn: im::OrdSet<usize>`
— the set of ability indices whose once-per-turn trigger already fired this turn. Doc: CR
603.2h. Reset to empty on zone change (CR 400.7 — new object; handled by `#[serde(default)]`
+ struct-literal defaults). Add `triggered_abilities_fired_this_turn: im::OrdSet::new()` to
EVERY explicit `GameObject { .. }` literal — same sites that set `skip_untap_steps: 0`
(builder.rs; effects/mod.rs token creation; resolution.rs ~L4592/4794/5510/6172/6385/6614).

**File**: `crates/engine/src/state/hash.rs` — GameObject hash (near L1292).
**Action**: `self.triggered_abilities_fired_this_turn.hash_into(hasher);`.

**File**: `crates/engine/src/rules/abilities.rs` — `flush_pending_triggers` (L6507). **This
is the once-per-turn GATE (the mutable point where triggers become stack objects).**
**Action**: For each `trigger` being flushed, look up its ability's `once_per_turn` via the
source's layer-resolved `triggered_abilities[trigger.ability_index]` (fall back to
`card_registry` like the existing `TriggeredAbility` resolution path). If `once_per_turn`:
- If `state.objects[source].triggered_abilities_fired_this_turn.contains(&ability_index)` →
  **skip** this trigger entirely (do not put on stack, ignore doublers).
- Else → put it on the stack exactly ONCE (force `additional_count = 0`, i.e. Panharmonicon
  cannot multiply a once-per-turn trigger), then
  `state.objects.get_mut(&source).triggered_abilities_fired_this_turn.insert(ability_index)`.
This handles BOTH within-batch multiples (e.g. 3 simultaneous creature deaths → first marks,
rest dropped) AND cross-turn (set cleared each untap step — Change 6). CR 603.2c/603.2h.

*(Do NOT try to gate inside `check_triggers` — it takes `&GameState` and cannot mutate. The
read-only skip there is optional and omitted to keep a single source of truth.)*

### Change 5: `KeywordAbility::DoesNotUntap` static + untap-step enforcement

**File**: `crates/engine/src/state/types.rs` — `KeywordAbility` enum (L394; add near
`CantBlock` L1665).
**Action**: Add `DoesNotUntap` with a doc comment: engine-internal representation of the
static "This permanent doesn't untap during its controller's untap step" (CR 502.3); NOT a
CR keyword; mirrors `CantBlock`. **Follow `KeywordAbility::CantBlock` (disc 160) as the
template** — grep every match arm for `KeywordAbility::CantBlock` and add a parallel
`DoesNotUntap` arm. The complete set of exhaustive-match sites (from the CantBlock grep):
- `crates/engine/src/state/hash.rs:922` → add `KeywordAbility::DoesNotUntap => 162u8.hash_into(hasher)`.
- `tools/replay-viewer/src/view_model.rs:885` (keyword display) → add
  `KeywordAbility::DoesNotUntap => "Doesn't Untap".to_string(),`.
- **No tui arm needed** — the tui `stack_view.rs` exhaustive match is on `StackObjectKind`,
  not `KeywordAbility` (CantBlock has no tui arm). Confirm with `cargo build --workspace`.

**File**: `crates/engine/src/rules/turn_actions.rs` — `untap_active_player_permanents`
(the untap loop at L1204–1226).
**Action**: Before the `else if obj.skip_untap_steps > 0` / `else if obj.status.tapped`
branch, add a layer-resolved check that skips untapping (and does NOT decrement
skip_untap_steps):
```rust
let chars = crate::rules::layers::calculate_characteristics(state, *id)
    .unwrap_or_else(|| state.objects.get(id).unwrap().characteristics.clone());
if chars.keywords.contains(&KeywordAbility::DoesNotUntap) {
    // CR 502.3: this permanent is kept from untapping. Do nothing (no untap, no decrement).
} else if obj.skip_untap_steps > 0 { ... } else if obj.status.tapped { ... }
```
CR 502.3: "effects can keep one or more of a player's permanents from untapping." Using
layer-resolved keywords means Humility/Dress Down (RemoveAllAbilities) correctly lets a
creature Sharpshooter untap again (edge-case test below). Watch the borrow: compute `chars`
before `state.objects.get_mut(id)` or restructure to avoid an aliasing conflict.

### Change 6: per-turn reset of the once-per-turn fired set

**File**: `crates/engine/src/rules/layers.rs` — `expire_until_next_turn_effects` (L1400;
the all-objects reset loop at L1433–1447 that zeroes `abilities_activated_this_turn`).
**Action**: In the same sweep, clear `triggered_abilities_fired_this_turn` for every
battlefield object whose set is non-empty. CR 603.2h: "once each turn" resets at the start
of each turn (this runs at every untap step, for all objects — the correct cadence, matching
the existing activated-ability once-per-turn reset).

### Change 7: exhaustive-match / hash sweep (the #1 compile-error source)

| File | Match / site | Action |
|------|--------------|--------|
| `state/hash.rs` L5181 (Effect) | add `UntapAll` arm | disc **87**, hash `filter` |
| `state/hash.rs` L4925 (TriggerCondition) | add 2 arms | `WheneverPermanentUntaps`=**42** (hash `filter`), `WhenCounterPlaced`=**43** (hash `counter`, `filter`, `on_self`) |
| `state/hash.rs` L2353 (TriggerEvent) | add 2 arms | `AnyPermanentUntaps`=**45**, `CounterPlaced`=**46** |
| `state/hash.rs` L922 (KeywordAbility) | add arm | `DoesNotUntap`=**162** |
| `state/hash.rs` L585-context (TriggeredAbilityDef hash) | hash new fields | `once_per_turn`, `counter_filter`, `counter_on_self` |
| `state/hash.rs` L1292-context (GameObject hash) | hash new field | `triggered_abilities_fired_this_turn` |
| `state/hash.rs` L202 | bump | `HASH_SCHEMA_VERSION = 28` |
| `state/hash.rs` parity test | update | `assert_eq!(HASH_SCHEMA_VERSION, 28)` |
| `tools/replay-viewer/src/view_model.rs` L885 (KeywordAbility display) | add arm | `DoesNotUntap => "Doesn't Untap"` |
| `tools/replay-viewer/src/view_model.rs` (Effect / TriggerCondition display, if exhaustive) | check & add | verify with `cargo build --workspace` |
| `tools/tui/src/play/panels/stack_view.rs` | verify | `StackObjectKind` match — no new SOK variant here (triggers reuse existing `TriggeredAbility`/`KeywordTrigger` SOK), so likely NO change; confirm build |

**After the implement phase run `cargo build --workspace` (NOT just `-p mtg-engine`)** to
catch the replay-viewer exhaustive matches — the runner misses `view_model.rs` ~50% of the
time (per `gotchas-infra.md`).

---

## Card Definition Fixes (backfill roster)

Grouped by the primitive that unblocks them. **TODO sweep result** (mandatory
roster-recall gate, run 2026-07-07): the greps below matched TODO/ENGINE-BLOCKED clauses
naming these five primitives; every card listed is a confirmed self-identified consumer.

### FULLY CLEAN after PB-AC1 (author all clauses, delete every TODO/ENGINE-BLOCKED)

| File | Primitive(s) | Fix |
|------|--------------|-----|
| `cards/defs/morbid_opportunist.rs` | once_per_turn | Add `Triggered { WheneverCreatureDies { exclude_self:true }, effect: DrawCards(1), once_per_turn: true }`. Delete ENGINE-BLOCKED note. |
| `cards/defs/mesmeric_orb.rs` | WheneverPermanentUntaps | `Triggered { WheneverPermanentUntaps { filter: None }, effect: MillCards { player: ControllerOf(Box::new(TriggeringCreature)), count: Fixed(1) } }`. (`TriggeringCreature` resolves via `entering_object_id` regardless of type — note the name is legacy.) |
| `cards/defs/goblin_sharpshooter.rs` | DoesNotUntap | Add `Keyword(DoesNotUntap)`; keep existing dies→UntapPermanent(Source) and {T}:damage. Delete "static restriction not in DSL" TODO. |
| `cards/defs/sky_hussar.rs` | UntapAll | Replace both `DrawCards(0)` placeholders (ETB + Forecast) with `UntapAll { filter: TargetFilter { card_types:[Creature], controller: You, ..default } }`. Delete TODOs. |
| `cards/defs/dusk_legion_duelist.rs` | WhenCounterPlaced + once_per_turn | `Triggered { WhenCounterPlaced { counter: Some(PlusOnePlusOne), on_self:true, filter:None }, effect: DrawCards(1), once_per_turn: true }`, keep Vigilance. |
| `cards/defs/sharktocrab.rs` | WhenCounterPlaced (+ existing tap/PreventNextUntap) | Add `Triggered { WhenCounterPlaced { counter: Some(PlusOnePlusOne), on_self:true }, effect: Sequence[TapPermanent(DeclaredTarget 0), PreventNextUntap(DeclaredTarget 0)], targets: [target creature an opponent controls] }`. Keep Adapt. |
| `cards/defs/whispering_wizard.rs` | once_per_turn | Set `once_per_turn: true` on the existing `WheneverYouCastSpell{noncreature}`→CreateToken trigger. Delete TODO. (Already fully authored otherwise.) |
| `cards/defs/fathom_mage.rs` | WhenCounterPlaced | `Triggered { WhenCounterPlaced { counter: Some(PlusOnePlusOne), on_self:true }, effect: DrawCards(1) }`, keep Evolve. **Caveat**: oracle says "you **may** draw" — if no optional-effect wrapper exists in the DSL, author as mandatory draw and leave a `// TODO(optional-draw)` (fidelity nit, not wrong-game-state; the "may" matters only for an empty library). Confirm whether an optional wrapper exists before deciding. |

### PARTIAL after PB-AC1 (author the now-expressible clause; leave a precise `// ENGINE-BLOCKED` on the residual clause — do NOT delete the whole marker)

| File | Now expressible | Residual (still blocked) |
|------|-----------------|--------------------------|
| `cards/defs/mana_vault.rs` | DoesNotUntap keyword | "you may pay {4} to untap" (optional cost → PB-AC2); "draw-step, if tapped, deal 1 to you" (draw-step trigger + tapped intervening-if) |
| `cards/defs/benefactors_draught.rs` | `UntapAll { creatures, Any }` | "until EOT, whenever an opponent's creature blocks, draw" (temporary opponent-block trigger) |
| `cards/defs/simic_ascendancy.rs` | `WhenCounterPlaced { filter: creature you control, on_self:false }` | "put THAT MANY growth counters" (dynamic amount = count from the triggering event — no `EffectAmount` for it) + "20+ growth = you win" (→ PB-AC8) |
| `cards/defs/baron_bertram_graywater.rs` | once_per_turn on the ETB-token trigger | "one or more **tokens** you control enter" — the any-token (non-creature) filter is an approximation via creature-ETB; broaden later |
| `cards/defs/kishla_skimmer.rs` | once_per_turn | "whenever a card leaves your graveyard" trigger |
| `cards/defs/esper_sentinel.rs` | once_per_turn | opponent-cast noncreature filter + "unless pays {X}" tax rider (→ PB-AC2) |
| `cards/defs/najeela_the_blade_blossom.rs` | `UntapAll { attacking creatures }` (the untap clause) | additional-combat-phase + Warrior-token attack trigger wiring |
| `cards/defs/bear_umbra.rs` | `UntapAll { lands you control }` on attack | (attack-trigger wiring — verify `WheneverCreatureYouControlAttacks`/self-attacks path covers Aura-host) |
| `cards/defs/moraug_fury_of_akoum.rs` | `UntapAll` untap clause | main-phase intervening-if + additional combat phase |
| `cards/defs/combat_celebrant.rs` | `UntapAll { other creatures you control }` | Exert + additional combat phase |

### OUT OF SCOPE — do NOT claim these (different mechanism)

- `cards/defs/seedborn_muse.rs`, `cards/defs/quest_for_renewal.rs`,
  `cards/defs/natures_will.rs` — "untap all permanents you control **during each other
  player's untap step**" is a STATIC that grants untapping during OTHER players' untap
  steps, not a one-shot `Effect::UntapAll`. Needs a distinct primitive (untap-step
  augmentation). Leave their TODOs; flag as a future micro-PB.
- `cards/defs/snap.rs`, `cards/defs/cloud_of_faeries.rs`, `cards/defs/frantic_search.rs`,
  `cards/defs/rewind.rs` — "untap up to N **target** lands" is targeted multi-untap
  (player choice), NOT filter-all `UntapAll`. Out of scope; needs an untap-N-targets
  effect. Leave TODOs.
- `cards/defs/charismatic_conqueror.rs` — "enters **untapped**" is an ETB-state trigger,
  not "becomes untapped" (CR 603.2e distinguishes them). Out of scope.

---

## New Card Definitions

None. PB-AC1 is a backfill batch — all affected cards already have def files.

---

## Unit Tests

**File**: `crates/engine/tests/pb_ac1_untap_counter.rs` (new). Follow the black-box builder
style in `tests/` (e.g. `evasion_protection.rs`, `card_def_fixes.rs`). Every test cites CR.

- `test_untap_all_untaps_matching_tapped_permanents` — CR 701.26b: `Effect::UntapAll`
  untaps tapped creatures you control, leaves opponents' creatures and lands tapped
  (filter scoping). Assert a non-tapped permanent is unaffected (no spurious event).
- `test_untap_all_only_tapped` — CR 701.26b: an already-untapped matching permanent emits
  no `PermanentUntapped` event (guard on `status.tapped`).
- `test_wheneverpermanentuntaps_fires_on_effect_untap` — CR 502.3/603.2: Mesmeric Orb; a
  permanent untapped by an effect makes its controller mill 1 (via
  `ControllerOf(TriggeringCreature)`).
- `test_wheneverpermanentuntaps_fires_at_untap_step_held_to_upkeep` — CR 502.4: untap-step
  untaps queue the trigger; it goes on the stack at upkeep (advance turn, drain).
- `test_wheneverpermanentuntaps_not_on_enters_untapped` — CR 603.2e: a permanent entering
  untapped does NOT fire the trigger (no untap event).
- `test_whencounterplaced_self_plus1plus1` — CR 122.6: Fathom Mage/Dusk Legion draws when a
  +1/+1 counter is placed on it; does NOT fire for a counter placed on another creature
  (on_self).
- `test_whencounterplaced_filtered_you_control` — CR 122.6: Simic Ascendancy-style
  (`on_self:false`, controller You) fires when a +1/+1 counter lands on another creature you
  control, not on an opponent's.
- `test_whencounterplaced_wrong_counter_kind_no_fire` — CR 122.7: a `-1/-1` (or loyalty)
  counter does NOT fire a `counter: Some(PlusOnePlusOne)` trigger.
- `test_once_per_turn_trigger_fires_once_across_turn` — CR 603.2h: Morbid Opportunist; two
  separate creature deaths in one turn draw exactly ONE card; resets next turn (advance a
  full turn cycle, kill again → draws again).
- `test_once_per_turn_trigger_batched_deaths` — CR 603.2c: three simultaneous creature
  deaths → exactly ONE trigger (flush-time within-batch dedup).
- `test_once_per_turn_not_multiplied_by_panharmonicon` — CR 603.2d vs 603.2h: a once-per-turn
  trigger fires once even with a trigger doubler present (use an ETB once-per-turn synthetic
  or document N/A if no doubler applies to the chosen event; if awkward, cover via a direct
  flush-path assertion instead).
- `test_does_not_untap_static_keeps_permanent_tapped` — CR 502.3: Mana Vault tapped, advance
  to its controller's untap step, assert still tapped.
- `test_does_not_untap_removed_by_humility` — CR 502.3 + 613.1f: a Goblin-Sharpshooter-like
  creature with `DoesNotUntap` under a Humility-style `RemoveAllAbilities` DOES untap
  (layer-resolved keyword check). *(This is the wedge test proving Change 5 uses
  `calculate_characteristics`, per `gotchas-rules.md #39` — the wedge property is the
  keyword itself, variable via ability removal.)*
- `test_does_not_untap_does_not_consume_skip_untap_counter` — CR 502.3: a permanent with
  both `DoesNotUntap` and a `skip_untap_steps` freeze does not decrement the freeze counter
  (static check precedes the decrement branch).
- `test_untap_all_multiplayer_controller_scope` — CR 502.3: 4-player; `UntapAll { You }`
  untaps only the caster's permanents.

**Also**: at least one integration test per fully-clean card that exercises the real
CardDefinition through `enrich_spec_from_def` (Morbid Opportunist, Mesmeric Orb, Goblin
Sharpshooter, Sharktocrab) — proves the card-def→runtime conversion loops wired the fields.

---

## Verification Checklist

- [ ] Engine primitives compile (`cargo check -p mtg-engine`)
- [ ] `once_per_turn: false` added to every explicit `AbilityDefinition::Triggered` literal
      in `cards/defs/` + builder/harness (grep `TriggerCondition::`)
- [ ] `triggered_abilities_fired_this_turn: im::OrdSet::new()` added to every explicit
      `GameObject { .. }` literal (grep the `skip_untap_steps:` sites)
- [ ] All 8 fully-clean card defs authored; every TODO/ENGINE-BLOCKED for the shipped
      primitives removed
- [ ] Partial card defs: now-expressible clause authored; residual clause left with a
      precise `// ENGINE-BLOCKED` naming the missing primitive
- [ ] `HASH_SCHEMA_VERSION` bumped to 28 + parity test updated
- [ ] Unit tests pass (`cargo test --all`)
- [ ] Clippy clean (`cargo clippy -- -D warnings`) — watch `large_enum_variant` on new
      `Effect`/`TriggerCondition` fields (box if needed)
- [ ] **Workspace builds (`cargo build --workspace`)** — confirms replay-viewer keyword arm
- [ ] `python3 tools/authoring-report.py` rerun; coverage delta posted
- [ ] No phantom `.claude/skills/*/SKILL.md` deletions committed (hazard #3)

---

## Risks & Edge Cases

- **once-per-turn gate location.** Gating in `flush_pending_triggers` (mutable, APNAP) — not
  `check_triggers` (immutable) — is load-bearing. It must (a) skip if already fired, (b) mark
  on first flush, (c) force `additional_count=0` so doublers can't multiply. Getting the
  order wrong re-introduces multi-fire. Cover with the batched-deaths and Panharmonicon tests.
- **Ability-index stability for the fired set.** The key is the index into layer-resolved
  `triggered_abilities`. If a mid-turn granted/removed ability shifts indices, the cap could
  mis-key. All roster cards have a single once-per-turn trigger (index 0 after keywords),
  so low risk; document the limitation. (Same simplification class as the shared
  `abilities_activated_this_turn` counter.)
- **`DoesNotUntap` as a pseudo-keyword.** Follows the `CantBlock` precedent, so Humility
  removal is free — but a hypothetical "gains all keyword abilities" effect could wrongly
  grant it. No such card in the game corpus; document and accept. If a reviewer objects,
  the fallback is a `LayerModification` + `Characteristics` bool (heavier; needs a
  full-dispatch layer test per `conventions.md`).
- **Untap-step borrow conflict.** Computing `calculate_characteristics(state, id)` needs an
  immutable borrow while the loop wants `state.objects.get_mut(id)`. Compute `chars`
  (and the `DoesNotUntap` bool) BEFORE taking the mutable borrow, or collect a
  skip-set first, then mutate.
- **CR 122.6 enters-with-counters.** `WhenCounterPlaced` will only fire if `CounterAdded` is
  emitted for counters an object enters with. If the enter-with-counters path does not emit
  `CounterAdded`, "put on as it enters" won't trigger. Verify and add a test asserting
  current behavior; do not expand scope to fix emission here.
- **Mesmeric Orb is global and unfiltered** — it fires for EVERY untap on EVERY player's
  untap step (4 dispatches per untap-step batch in a 4-player game, plus effect untaps).
  Confirm no performance cliff (the dispatch is O(battlefield) per untapped permanent). The
  existing `collect_triggers_for_event(only_object=None)` already scans all permanents; this
  is consistent with `AnyCreatureDies`.
- **`WheneverCreatureDies` for Morbid Opportunist** must use `exclude_self: true` ("other
  creatures") — but the 2024-11-08 ruling says it still triggers if Morbid dies alongside
  others (look-back). The existing death path handles LKI; verify the exclude_self+batch
  interaction in the once-per-turn-across-turn test.
- **Yield honesty.** Confirmed fully-clean ≈ 8 (not the advisory ~22). Partials produce
  fidelity but not clean-coverage movement until their residual PBs (AC2/AC8) ship — expect
  the authoring-report delta to move by ~8 clean, matching the recalibration in
  `campaign-plan-2026-05-16.md §0`.
