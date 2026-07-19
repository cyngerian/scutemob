# Primitive Batch Plan: PB-OS4b — Face-Aware Ability Gathering for Transformed Permanents

**Generated**: 2026-07-19
**Primitive**: A transformed permanent (`obj.is_transformed == true`) must gather its **back
face's** non-keyword abilities (triggered / activated / static / mana) and must **not** use its
**front** face's. Behavior-only correctness fix — branches on the already-existing `is_transformed`
state; adds no new DSL type, no `Effect` variant, no struct field.
**Seed**: OOS-OS4-2 (`memory/primitives/ef-batch-plan-2026-07-17.md` §13; evidence
`memory/primitives/pb-review-OS4.md` E1/C1).
**CR Rules**: 712.8d/712.8e (back-face-up permanent has only that face's characteristics — and, by
extension, its abilities), 712.8a (front face outside battlefield), 400.7 (is_transformed reset on
zone change → off-battlefield reads are front automatically), 714.4 (transformed Saga is no longer a
Saga), 306.5b (loyalty — only if the deferred edgar half is taken).
**Cards affected**: 5 mandatory (2 shipped-`Complete`-but-currently-wrong + 3 `Partial`) + 1 deferred
(`edgar_charmed_groom`, optional half). Plus incidental cleanup of front-ability leaks on
`beloved_beggar` / `brutal_cathar` (no marker change).
**Dependencies**: PB-EF5 (in-place `TransformSelf`, shipped), PB-OS4 (`ExileSourceAndReturnTransformed`
+ enter-transformed object construction, shipped on this branch at PROTOCOL 19 / HASH 56).
**Deferred items from prior PBs**: OOS-OS4-2 (this PB). OOS-OS4-1 (planeswalker back-face loyalty)
stays deferred — not touched here.
**Wire expectation**: **mandatory scope = NO bump** (stays PROTOCOL 19 / HASH 56). **edgar half (if
taken) = exactly one bump** (PROTOCOL 19→20, HASH 56→57), in a **separate commit**.
**TODO sweep (roster-recall gate)**: ran `OOS-OS4-2 | face-aware | back-face abilit | back-face
activated/static/trigger` over `crates/card-defs/src/defs/`. **1 card self-identifies the primitive**:
`fable_of_the_mirror_breaker.rs` (lines 24-25, 117-122, 184-185) — already in scope. No other card
names the gap. The broader affected roster was found structurally (DFC back faces with non-keyword
abilities), see below.

---

## Primitive Specification

Two distinct ability-representation channels exist in the engine, and **both** are currently
front-face-only for transformed permanents:

- **Channel A — runtime characteristics vectors.** `Characteristics` carries
  `triggered_abilities: Vec<TriggeredAbilityDef>`, `activated_abilities: Vec<ActivatedAbility>`,
  `mana_abilities: Vector<ManaAbility>`. These are lowered from `def.abilities` at object
  construction (the three loops in `enrich_spec_from_def`, `testing/replay_harness.rs:2131-2311`),
  stored on `obj.characteristics`, and read either directly (`obj.characteristics.<vec>`) or via
  `calculate_characteristics`/`expect_characteristics` (which clone the base and layer grants on
  top). **`transform_permanent_in_place` (`rules/engine.rs:1124`) flips `is_transformed` and does
  NOT rebuild these vectors**, and the layer back-face substitution
  (`rules/layers.rs:97-139`) rewrites name/types/keywords/P-T/colors but **leaves the three ability
  vectors untouched**. Net: after transform, Channel A still holds the **front** face's
  triggered/activated/mana abilities. This is why `bloodline_keeper`'s front `{B}: transform`
  activated ability stays activatable and its back `Lord of Lineage` abilities are absent, and why
  `docent_of_perfection`'s front cast-trigger keeps firing while the back cast-trigger never does.

- **Channel B — raw `def.abilities` direct-index.** Several sites iterate `def.abilities`
  (front) directly and dense-index it by a stored `ability_index` (e.g.
  `PendingTriggerKind::CardDefETB`), never consulting `back_face`. These are: static-continuous
  registration, `CardDefETB` ETB-trigger queueing, the upkeep triggered-ability sweep, the
  tapped-for-mana trigger sweep, and the Saga final-chapter SBA. The **only** back-face ability
  reader in the whole engine today is `layers.rs:116`, and it copies **keywords only**.

The fix makes both channels face-aware:

- **Channel A** is fixed at the transform boundary: whenever `is_transformed` flips (or an object is
  constructed already transformed), rebuild `obj.characteristics.{triggered,activated,mana}_abilities`
  from the **effective face's** abilities using the same lowering logic as `enrich_spec_from_def`.
  Because base is rebuilt at exactly the flip, base == layer-resolved for these vectors, so **every**
  Channel-A reader (direct-base OR `calculate_characteristics`-based) becomes correct with **zero
  per-reader edits**. (See "Mechanism design" for why this beats rebuilding inside
  `calculate_characteristics`.)

- **Channel B** is fixed with a new `CardDefinition::effective_abilities(is_transformed)` helper
  (card-types) routed through each battlefield-permanent def-index site, on **both** the producer
  (scan/queue) and the consumer (resolution), each gated on the object's `is_transformed`.

Casting / alt-cost / partner / copy / spell-on-stack lookups that read the **printed front face**
regardless of transform state must NOT change (enumerated below).

---

## CR Rule Text (from MCP)

**712.8d** — "While a double-faced permanent has its front face up, it has only the characteristics
of its front face."
**712.8e** — "While a nonmodal double-faced permanent has its back face up, it has only the
characteristics of its back face. However, its mana value is calculated using the mana cost of its
front face…" ("characteristics" includes abilities per CR 109.3 / 604 — the abilities a permanent
has are part of what its face defines.)
**712.8a** — "While a double-faced card is … in a zone other than the battlefield or stack, it has
only the characteristics of its front face."
**400.7** — "An object that moves from one zone to another becomes a new object with no memory of …
its previous existence." (Engine resets `is_transformed=false` on every zone change, so
off-battlefield/library/graveyard/exile/hand reads are front-face automatically — no code needed for
712.8a.)
**714.4** — a Saga is sacrificed by SBA only while it is a Saga with lore ≥ final chapter; a
permanent that has transformed to its non-Saga back face is no longer a Saga (so the Saga SBA must
not see its front chapters once transformed).

---

## Engine Changes

### Change 1 — `CardDefinition::effective_abilities` helper (Channel B)

**File**: `crates/card-types/src/cards/card_definition.rs`
**Action**: add an inherent method:

```rust
impl CardDefinition {
    /// CR 712.8d/e: the abilities of this permanent's currently-visible face.
    /// Back face when transformed and a back face exists; otherwise the front list.
    /// Off-battlefield objects always have is_transformed=false (CR 400.7 / 712.8a),
    /// so this returns the front list there automatically.
    pub fn effective_abilities(&self, is_transformed: bool) -> &[AbilityDefinition] {
        match (is_transformed, self.back_face.as_ref()) {
            (true, Some(face)) => &face.abilities,
            _ => &self.abilities,
        }
    }
}
```

**SR-6 note**: card-defs depend on card-types only. This method reads only `CardDefinition` /
`CardFace` (both already in card-types), so placing it here is legal and keeps the 1,798 card defs
`Fresh` when the engine changes. Confirmed no such method exists today.

### Change 2 — Channel-A face-aware runtime-vector rebuild at the transform boundary

**File (new helper, engine crate)**: `crates/engine/src/rules/` (recommend a small function in
`rules/abilities.rs` or a new `rules/face.rs`), plus refactor of `enrich_spec_from_def`.

**Action**:

1. Extract the three lowering loops now inlined in `enrich_spec_from_def`
   (`testing/replay_harness.rs:2131-2311` — the mana-ability loop, the non-mana activated-ability
   loop, and the triggered-ability loop that populates `triggering_creature_filter`, `targets`,
   `once_per_turn`, `modes`, ForEach handling, etc.) into a shared, non-test engine function:

   ```rust
   pub(crate) fn build_face_ability_vectors(
       abilities: &[AbilityDefinition],
   ) -> (Vector<ManaAbility>, Vec<ActivatedAbility>, Vec<TriggeredAbilityDef>);
   ```

   `enrich_spec_from_def` then calls this for the front face (behavior byte-identical — verify no
   test diff). **Preserve** the `mana_ability_lowering` / `cost_to_activation_cost` disjointness
   (SR-34/SF-6): an ability lowered into `mana_abilities` must be excluded from `activated_abilities`
   at the same index discipline. **Exclude** the ObjectSpec-level attach/detach (equipment) entries
   at `replay_harness.rs:2244-2274` from the shared helper — those are not `def.abilities`-derived
   and are equipment-only (DFCs are not equipment; a transformed permanent uses its back-face list,
   which has no such synthetic entries).

2. Add `pub(crate) fn apply_face_change(state, obj_id, new_is_transformed: bool)` (see Change 3) that
   calls `build_face_ability_vectors(def.effective_abilities(new_is_transformed))` and writes the
   three vectors onto `obj.characteristics`.

**Why base-rebuild and not `calculate_characteristics` rebuild**: `calculate_characteristics` is the
"single source of truth" for keywords/types, but the three ability vectors have **direct-base
readers** that bypass it — notably `resolution.rs:1856` (activated-ability resolution reads
`obj.characteristics.activated_abilities`) and the triggered scans at `abilities.rs:6610` /
`4371` / `4727` / `4827` (base `obj.characteristics.triggered_abilities`). Rebuilding only inside
`calculate_characteristics` would leave those base-readers front-faced → an index/identity mismatch
between validation (layer-resolved) and resolution (base). Rebuilding the **base** at the boundary
makes base == layer-resolved for these vectors, so **all** readers are correct with no reader
auditing — avoiding the exhaustive-match miss that is this project's #1 failure mode. `im`-rs base
mutation at transform is already how `is_transformed`, `last_transform_timestamp`, keywords-via-layer
behave; no new state-shape.

**CR**: 712.8d/e.

### Change 3 — Route every transform boundary through one `apply_face_change`

**File**: `crates/engine/src/rules/` (helper) + the 8 mutation sites.
**Action**: introduce `apply_face_change(state, obj_id, new_is_transformed)` that, in order:

1. reads `def` + old `is_transformed`; early-return if unchanged or non-DFC;
2. **deregisters** the *old* face's static continuous effects for this source (see Change 4);
3. sets `obj.is_transformed = new_is_transformed`;
4. rebuilds `obj.characteristics.{mana,activated,triggered}_abilities` from
   `def.effective_abilities(new_is_transformed)` via `build_face_ability_vectors`;
5. **registers** the *new* face's static continuous effects (see Change 4).

Then route the **flip / enter-transformed** sites through it (do NOT touch the many
`is_transformed: false` construction defaults — those are already front and correct):

| # | Site | Context | Action |
|---|------|---------|--------|
| 1 | `rules/engine.rs:1209` | `transform_permanent_in_place` in-place flip | replace the raw flip with `apply_face_change` (flag already toggled there; refactor to compute new value then call) |
| 2 | `rules/engine.rs:1433` | craft return (enters transformed) | after construction, `apply_face_change(state, id, true)` |
| 3 | `effects/mod.rs:4292` | `ExileSourceAndReturnTransformed` (enters transformed) | after the `obj.is_transformed=true` block, `apply_face_change(state, battlefield_id, true)` **before** the existing `register_static_continuous_effects` / `queue_carddef_etb_triggers` calls (and make those two use `effective_abilities`, Change 5 — the register call inside `apply_face_change` supersedes the standalone one; remove the now-duplicate standalone `register_static_continuous_effects` here) |
| 4 | `rules/turn_actions.rs:1648` | daybound/nightbound day↔night flip | route through `apply_face_change` |
| 5 | `rules/resolution.rs:665` | verify context (Disturb / cast-transformed) | route through `apply_face_change` |
| 6 | `rules/resolution.rs:7173` | `obj_mut.is_transformed = to_back_face` | route through `apply_face_change` |
| 7 | `rules/resolution.rs:7214` | `obj.is_transformed = true` | route through `apply_face_change` |
| 8 | `rules/resolution.rs:7317` | `obj_mut.is_transformed = to_back_face` | route through `apply_face_change` |

**Runner must open each of sites 5-8 and confirm what triggers them** (Disturb cast, meld?, other
enter-transformed paths) before wiring — mislabeling risks a double register/deregister. The helper
is idempotent-safe (step 1 early-returns if unchanged).

### Change 4 — Static continuous effects: deregister-old / register-new at the boundary

**File**: `crates/engine/src/rules/replacement.rs` (extend `register_static_continuous_effects`) +
the `apply_face_change` helper.
**Problem**: `register_static_continuous_effects` runs once at ETB and pushes persistent
`ContinuousEffect` entries (source = obj, `duration: WhileSourceOnBattlefield`, `is_cda: false`).
In-place transform never re-runs it, so front statics persist (leak) and back statics never register.
`ContinuousEffect` has no origin-face tag and we may not add one (HASH-affecting → forbidden in
mandatory scope).
**Action**:

- **Register (new face)**: change `register_static_continuous_effects` to iterate
  `def.effective_abilities(is_transformed)` (add an `is_transformed: bool` param, or read it from the
  live object). Called from `apply_face_change` step 5 and (unchanged effect) from every ETB site
  (pass the entering object's `is_transformed`, which is `false` for normal ETB and `true` for the
  return/enter-transformed paths).
- **Deregister (old face)**: add `deregister_face_statics(state, obj_id, old_face_abilities)` — for
  each `AbilityDefinition::Static` in the *old* face, remove one matching entry from
  `state.continuous_effects` where `source == Some(obj_id)` and `(layer, duration, modification,
  resolved_filter)` structurally match (resolve `EffectFilter::Source → SingleObject(obj_id)` the
  same way `register` does before comparing). This is precise and does not disturb non-static
  effects the object may own. **Highest-risk sub-change** — pin with the front-static-removed decoy
  test (AC 5057).
- **Also handle `TriggerDoubling` and replacement abilities** in the same face-aware manner if a back
  face declares them (none in the current roster; make `register`/`deregister` iterate the full
  effective list so they are covered). Replacement abilities are registered by a *separate* helper
  (`register_permanent_replacement_abilities`) — audit whether it too reads `def.abilities`; if a
  back face can declare a replacement (none in roster), make it face-aware for symmetry or document
  the limitation.

**Roster note**: `docent_of_perfection` and `bloodline_keeper` fronts have **no** `Static`, so their
transform only needs the **register-new** path; the **deregister-old** path is exercised only by the
decoy test and by transform-back. Implement both regardless (correctness + AC 5057).

**CR**: 613 / 604 (statics function only while the ability's face is up).

### Change 5 — Channel-B def-index sites: route through `effective_abilities`

Each site below reads `def.abilities` for a **battlefield permanent** and must instead read
`def.effective_abilities(obj.is_transformed)`, on **both** producer and consumer. `ability_index`
values pushed onto `PendingTrigger` (kind `CardDefETB`) are dense indices into the **effective**
list; the consumer must re-derive against the effective list using the object's `is_transformed`
**at consume time** (see Index-Stability Analysis).

| Site | File:Line | Gathers | Action |
|------|-----------|---------|--------|
| static registration | `replacement.rs:2057` | `Static`/`TriggerDoubling` for ETB/transform | Change 4 (effective list) |
| ETB trigger queue | `replacement.rs:1415` (`queue_carddef_etb_triggers`) | `WhenEntersBattlefield`/`TributeNotPaid` | iterate `effective_abilities` (producer); index → effective list |
| upkeep sweep | `turn_actions.rs:277` | `AtBeginningOf{Your,Each}Upkeep` | iterate `effective_abilities`; index → effective list |
| tapped-for-mana sweep | `mana.rs:675` | `WhenTappedForMana` | iterate `effective_abilities` |
| Saga final-chapter SBA | `sba.rs:839` and `sba.rs:878` | `SagaChapter` (max chapter + pending-chapter guard) | read `effective_abilities`; a transformed Saga's back face has no `SagaChapter` → `final_chapter=None` → not a Saga → not sacrificed (CR 714.4). Pin with `test_saga_transform_not_sacrificed` |
| CardDefETB effect lookup | `abilities.rs:6002` | resolve `CardDefETB` effect by index | `effective_abilities(def, obj.is_transformed).get(ability_index)` |
| CardDefETB once-per-turn | `abilities.rs:6816` | `CardDefETB` once_per_turn flag | same |
| CardDefETB has-targets | `abilities.rs:6889` | `CardDefETB` target presence | same |
| CardDefETB target reqs | `abilities.rs:7012` | `CardDefETB` target requirements | same |
| CardDefETB (other) | `abilities.rs:8169` (`def.abilities.get(*ability_index)`) | inspect; if `CardDefETB` consumer → effective | confirm & convert |

**Do NOT change** the `PendingTriggerKind::Normal` branches (`abilities.rs:3874/4267/4371/4727/4827/
6610` and `mana.rs:714`): those index the **runtime** `characteristics.triggered_abilities` (Channel
A), already fixed by Change 2. Nor the `WhenYouCastThisSpell` scan at `abilities.rs:3622` (indexes the
**spell's** front def on the stack — a spell is front-face per 712.8c unless cast transformed, which
is a separate handled path).

### Change 6 — No exhaustive-match / wire updates

No new enum variant, no new struct field, no `HashInto` change, no `StackObjectKind`/`KeywordAbility`
arm, no `PROTOCOL_SCHEMA_FINGERPRINT` change. **Verify** `PROTOCOL_VERSION` stays 19
(`rules/protocol.rs:178`) and `HASH_SCHEMA_VERSION` stays 56 (`state/hash.rs:504`) — the mandatory
scope must leave both untouched (AC 5040). `cargo build --workspace` + the wire-fingerprint gate are
the check.

---

## Index-Stability Analysis (highest-risk aspect)

The danger: a stored `ability_index` produced against one face's list but consumed against the
other's → wrong ability dispatched or panic.

**Channel A (Normal triggers, activated abilities).** Producer and consumer both dense-index the
runtime `characteristics.<vec>`. With Change 2, base == effective face at all times, and the flip is
atomic (single `apply_face_change`). Activation: the player-chosen `ability_index` (validated via
`expect_characteristics`, `abilities.rs:219/231/312`) is consumed at the same instant to build the
stack object — `is_transformed` cannot change between validate and place. Resolution reads base
(`resolution.rs:1856`) which now equals the same effective list. **Consistent.**

**Channel B (`CardDefETB` triggers).** Producer (`queue_carddef_etb_triggers`, upkeep sweep) pushes
`ability_index` = index into `effective_abilities(def, is_transformed_at_queue)`. Consumer
(`abilities.rs:6002/6816/6889/7012`) re-derives with `effective_abilities(def,
is_transformed_at_consume)`. **These agree iff `is_transformed` is identical at queue and consume.**
For the intended roster this holds: e.g. an upkeep trigger queued while transformed is resolved while
still transformed (the trigger's own effect may transform the object **back**, but the effect is
looked up at resolution *start*, before it executes). **Residual hazard (document, accept):** if some
*other* effect transforms the object between queue and resolution, or if a single object queues
multiple back-face upkeep triggers and the first transforms it back before the second resolves, the
later index is stale. No roster card exercises this (Edgar — the only back-face-upkeep card — is
deferred and has a single trigger). We cannot record the face on `PendingTrigger` without a
HASH-affecting field, so the "`is_transformed` at consume" rule is the chosen contract; note it in a
code comment at each CardDefETB consumer.

**`activated_ability_cost_reductions` keying.** `get_self_activated_reduction`
(`abilities.rs:9042`) is keyed by the activated-ability index into `characteristics.activated_abilities`,
mapped to the **front**-`CardDefinition`-level `activated_ability_cost_reductions`. After Change 2 a
transformed permanent's `activated_abilities` are back-face-derived, so a back-face activated ability
at index 0 could collide with a front cost-reduction keyed at 0. **Mitigation**: when
`obj.is_transformed`, **skip** `get_self_activated_reduction` (the schema has no back-face cost
reductions; a back face cannot declare one). Add the `is_transformed` guard at `abilities.rs:725-734`.
No roster card has a back-face activated ability with a colliding front reduction, but the guard
closes the hazard.

---

## Mechanism Design — summary of decisions

- **Channel A**: `build_face_ability_vectors(&[AbilityDefinition])` (extracted from `enrich`,
  engine crate) + rebuild base at the boundary via `apply_face_change`. Chosen over
  rebuild-in-`calculate_characteristics` because direct-base readers exist; base-rebuild fixes all
  readers with zero reader-auditing. (Trade-off recorded above.)
- **Channel B**: `CardDefinition::effective_abilities(is_transformed)` (card-types) at each
  def-index battlefield site, producer + consumer, gated on live `is_transformed`.
- **Statics**: deregister-old + register-new at the boundary (structural match), since
  `ContinuousEffect` cannot carry a face tag without a HASH bump.
- **Single choke point**: `apply_face_change` performs deregister→flip→rebuild→register atomically
  and is the only place `is_transformed` is mutated for battlefield permanents (all 8 sites routed).

---

## Card Definition Fixes

### docent_of_perfection.rs — **probe, expect stays `Complete`**
Front: Flying, Transform, `Triggered(WheneverYouCastSpell → token; then if ≥3 Wizards TransformSelf)`.
Back (Final Iteration): Flying, 3× `Static` (Wizards +2/+1 & flying), `Triggered(WheneverYouCastSpell →
token)`. **Currently wrong** (Complete but front cast-trigger keeps firing post-transform, back
statics/trigger dead). After the fix: probe by executing a cast while transformed — assert the back
token trigger fires, the front "then transform" clause is gone, and the Wizard anthem applies. If
correct → keep `Complete` (pinned by tests). If still wrong → demote honestly.

### bloodline_keeper.rs — **probe, expect stays `Complete`**
Front: Flying, Transform, `Activated({T}→Vampire token)`, `Activated({B}→TransformSelf, if ≥5
Vampires)`. Back (Lord of Lineage): Flying, `Static(other Vampires +2/+2)`, `Activated({T}→token)`.
**Currently wrong.** After the fix: assert post-transform the back `{T}` token ability is activatable,
the front `{B}: transform` ability is **not** in the activated list, and the +2/+2 anthem applies.
Keep `Complete` if correct, else demote.

### growing_rites_of_itlimoc.rs — stays `Partial`; **message must become accurate**
Back (Itlimoc): 2× mana `Activated` (`{T}: {G}`, `{T}: {G} per creature`). Front had no mana ability.
**Currently wrong**: the partial note claims "the back face's two mana abilities are fully
implemented," but pre-fix they never function (transformed Itlimoc cannot tap for mana — Channel A
holds the empty front list). After the fix the claim becomes true. Add a test that taps transformed
Itlimoc for mana; keep `Partial` (front ETB "look at top four, take a creature" remains inexpressible,
OOS-EF5-4f). Verify/retighten the message so it does not assert function that only exists post-merge.

### thaumatic_compass.rs — stays `Partial`
Back (Spires of Orazca): `Activated({T}: {C})`, `Activated({T}: untap target attacking creature…)`.
**Currently wrong** (back activated abilities dead when transformed). After the fix they function;
the `Partial` reason (missing remove-from-combat primitive, OOS-EF5-4g) still holds. Add a test that
taps transformed Spires for `{C}`. Message unchanged.

### fable_of_the_mirror_breaker.rs — stays `Partial`; **correct the C2 message**
Back (Reflection of Kiki-Jiki): `Activated` (copy-a-creature). Correct the note at lines 117-122 and
184-185: after this PB the back-face activated ability **is reachable/activatable** — it is no longer
"never gathered." It remains `Partial` for the chapter-I token-trigger and chapter-II bounded-discard
DSL gaps only. Do **not** claim the copy ability is fully correct if the Kiki-Jiki `nonlegendary`
TargetFilter gap still applies — state precisely what works (activation reachable) vs. what does not.

### Incidental (no marker change, verify no regression)
`beloved_beggar.rs` (Disturb) — front `WhenDies(gain 3)` no longer leaks onto the Generous Soul back
(it exiles rather than dies, so unobservable, but the leak is cleaned). `brutal_cathar.rs`
(daybound/nightbound) — back is keyword-only; transform must remain a no-op for its ability set
(good decoy). `braided_net.rs` — back abilities are empty DSL-gap stubs; unaffected (note only).

---

## New Card Definitions

None in mandatory scope. **`edgar_charmed_groom.rs` only if the deferred half is taken** (see below).

---

## Unit Tests

**File**: `crates/engine/tests/mechanics_m_z/pb_os4b_face_aware_abilities.rs` (new; add `mod`
line to `crates/engine/tests/mechanics_m_z/main.rs` — SR-9a: a missing `mod` silently drops the
whole file).

Probe-by-execution (AC 5058) + face-distinction decoys (AC 5057). Each test cites CR 712.8d/e.

- `test_docent_back_cast_trigger_fires_after_transform` — transform docent (control ≥3 Wizards), cast
  an instant, assert exactly **one** Wizard token from the back trigger and **no** further TransformSelf
  (front "then transform" clause is gone).
- `test_docent_front_cast_trigger_stops_after_transform` — decoy pairing: same setup, assert the
  front trigger's conditional-transform does not re-fire.
- `test_docent_back_wizard_anthem_applies` — a Wizard gets +2/+1 & flying only after docent is
  transformed (front had no static → nothing before).
- `test_bloodline_back_token_ability_activatable_after_transform` + `_front_transform_ability_gone` —
  post-transform, `{T}` back ability activatable; front `{B}: transform` index absent (attempt →
  `InvalidAbilityIndex`).
- `test_bloodline_back_vampire_anthem_applies_after_transform`.
- `test_growing_rites_itlimoc_taps_for_mana_after_transform` — was dead pre-fix.
- `test_thaumatic_compass_spires_taps_for_colorless_after_transform`.
- `test_fable_reflection_activated_reachable_after_transform` — activation validates (reachable);
  does not assert full copy correctness (Kiki-Jiki filter gap).
- **Decoy** `test_front_static_removed_on_transform` — build a minimal DFC decoy whose **front** has a
  `Static` anthem and whose **back** has none; assert the anthem is gone post-transform (pins
  `deregister_face_statics`).
- **Decoy** `test_back_upkeep_trigger_fires_only_when_transformed` — decoy DFC: front has **no**
  upkeep trigger, back has `AtBeginningOfYourUpkeep`; assert nothing at upkeep while front, the
  trigger fires while transformed (pins `turn_actions.rs` effective-abilities + CardDefETB
  producer/consumer index parity).
- **Decoy** `test_transform_there_and_back_restores_front_ability_set` — transform, then transform
  back; assert front abilities return and back abilities/statics are gone (pins bidirectional
  deregister/register + index stability).
- `test_saga_transform_not_sacrificed` — a transforming Saga (Fable, or a decoy Saga→non-Saga) with
  lore ≥ final chapter is **not** sacrificed after it transforms (CR 714.4; pins `sba.rs`
  effective-abilities).
- **Negative** `test_non_dfc_transform_is_noop_ability_set` — a non-DFC never changes its ability set.
- **Negative** `test_offbattlefield_uses_front_abilities` — a DFC in graveyard/hand reports front
  abilities (is_transformed reset on zone change, CR 400.7/712.8a).

**Pattern**: follow `crates/engine/tests/mechanics_m_z/pb_os4_return_transformed.rs` (14 tests,
decoy-heavy) and the PB-EF5 TransformSelf tests. Reuse `enrich_spec_from_def` + `GameStateBuilder`
setup used there.

---

## Optional Extension — `edgar_charmed_groom` return-transformed lifecycle

**Recommendation: DEFER to a follow-up micro-PB (call it OOS-OS4-3).** Do **not** take it in this PB.

Justification:
1. The mandatory scope is already large and high-risk (two channels, static remove/readd, 8 boundary
   sites, index-stability). Keeping it **wire-neutral** (PROTOCOL 19 / HASH 56) makes it one clean,
   reviewable correctness commit — matching AC 5040 and the OS4 reviewer's own "no speculative
   machinery / one bump per PB" discipline.
2. `edgar_charmed_groom.rs` **does not exist** (it was left unauthored in the OS4 fix phase), and the
   effect it needs — `Effect::ReturnSourceToBattlefieldTransformed` (return **from graveyard**, not
   exile-and-return) — was **removed in the OS4 narrowing** (only `ExileSourceAndReturnTransformed`
   remains, `card_definition.rs:2108`). Re-adding a return-from-graveyard-transformed `Effect` variant
   is a genuine wire bump (PROTOCOL 19→20, HASH 56→57) and must be its own commit with the card that
   proves it — exactly the pattern the OS4 review mandated.
3. Edgar cannot ship `Complete` until this PB's face-aware gathering lands (its whole back-face Coffin
   loop depends on it). Sequencing it **after** the mandatory fix is merged/reviewed is cleaner than
   bundling an unrelated wire bump into a wire-neutral correctness PB.

**If the runner is nonetheless directed to take it** (only if edgar then ships `Complete` with a full
lifecycle test): it is a **separate commit** = one wire bump (PROTOCOL 19→20, HASH 56→57), re-adding
`Effect::ReturnSourceToBattlefieldTransformed` (executor + `HashInto` arm discriminant + protocol
History row + epoch row + re-pinned fingerprints/`FROZEN_HISTORY_PREFIX_DIGEST`/sentinels), authoring
`edgar_charmed_groom.rs` (front: Vampire anthem `Static` + `WhenDies → ReturnSourceToBattlefieldTransformed`;
back Edgar Markov's Coffin: `AtBeginningOfYourUpkeep → create lifelink Vampire token + bloodline
counter; if ≥3 counters remove them and transform back`), and a full-lifecycle test (dies → returns
as Coffin new object → upkeep makes a Vampire + counter → the Coffin does **NOT** grant the front
Vampire anthem → 3 counters transform back to front → front anthem returns). CR 306.5b is **not**
needed (Edgar's back is not a planeswalker); CR 400.7 (new object) + 714 n/a. File it as OOS-OS4-3 in
the seed ledger either way.

---

## Ordered Implementation Steps

1. **Change 1** — add `CardDefinition::effective_abilities` (card-types). `cargo check -p mtg-card-types`.
2. **Change 2a** — extract `build_face_ability_vectors` from `enrich_spec_from_def`; refactor `enrich`
   to call it for the front face. Run full tests — **must be a zero-diff refactor**.
3. **Change 2b/3** — add `apply_face_change` (deregister→flip→rebuild→register); route the 8 boundary
   sites through it (confirm sites 5-8 context first).
4. **Change 4** — make `register_static_continuous_effects` face-aware (`effective_abilities`) and add
   `deregister_face_statics`; wire both into `apply_face_change` and the ETB call sites (pass
   `is_transformed`).
5. **Change 5** — route the Channel-B def-index sites (ETB queue, upkeep sweep, mana sweep, Saga SBA,
   CardDefETB consumers) through `effective_abilities`; add the `is_transformed` guard on
   `get_self_activated_reduction`; add code comments recording the "is_transformed at consume" contract.
6. **Probe tests** — docent, bloodline, growing_rites, thaumatic_compass, fable (AC 5058). Confirm
   docent/bloodline stay `Complete`; if not, demote and record why.
7. **Decoy tests** — front-static-removed, back-upkeep-only, there-and-back, saga-not-sacrificed,
   non-DFC-noop, off-battlefield-front (AC 5057).
8. **Card-def updates** — verify docent/bloodline markers; correct fable C2 message; retighten
   growing_rites message; leave thaumatic_compass.
9. **Docs** — mark OOS-OS4-2 resolved in `memory/primitives/oos-retriage-plan-2026-07-18.md` and
   `memory/primitives/ef-batch-plan-2026-07-17.md` §13; note affected-card audit outcome. (CLAUDE.md
   Current State delta handled at `/collect`.)
10. **Gates** — `cargo test --all`; `cargo clippy -- -D warnings`; `cargo fmt --check` **and**
    `tools/check-defs-fmt.sh`; `cargo build --workspace`; **assert PROTOCOL 19 / HASH 56 unchanged**.
11. **(Deferred)** file OOS-OS4-3 for the edgar half.

---

## Verification Checklist

- [ ] `effective_abilities` compiles in card-types; card defs stay `Fresh` (SR-6).
- [ ] `build_face_ability_vectors` extraction is a zero-diff refactor of `enrich`.
- [ ] All 8 `is_transformed` flip/enter sites routed through `apply_face_change`; no raw flip remains.
- [ ] Static deregister-old / register-new correct for both transform and transform-back.
- [ ] All Channel-B def-index battlefield sites use `effective_abilities`; front-only sites untouched.
- [ ] `get_self_activated_reduction` guarded on `is_transformed`.
- [ ] docent + bloodline verified `Complete` by execution (or honestly demoted).
- [ ] fable C2 message corrected; growing_rites message accurate; both stay `Partial`.
- [ ] New test file has its `mod` line; all decoys + probes pass.
- [ ] `cargo test --all`, clippy `-D warnings`, `cargo fmt --check` + `check-defs-fmt.sh`,
      `build --workspace` all green.
- [ ] **PROTOCOL_VERSION == 19 and HASH_SCHEMA_VERSION == 56 (no wire change).**
- [ ] No remaining TODO in touched card defs.

---

## Risks & Edge Cases

- **Static deregister structural match** (Change 4) is the trickiest piece — must resolve
  `EffectFilter::Source` before comparing, and remove exactly one matching entry per old-face static.
  A miss leaves a leaked front anthem (the exact C1 bug). Pinned by `test_front_static_removed_on_transform`.
- **CardDefETB index parity across a mid-resolution transform** — accepted residual hazard (documented
  contract: index re-derived against `is_transformed` at consume). Only reachable by a back-face
  upkeep/ETB trigger that transforms its own object then has another queued back-face trigger resolve;
  no non-deferred roster card hits it.
- **`enrich` extraction fidelity** — `build_face_ability_vectors` must reproduce
  `triggering_creature_filter`/`targets`/`once_per_turn`/`modes`/ForEach handling exactly, or Normal
  cast-triggers (docent) mis-fire. Guard: step 2 must be a zero-diff refactor before any face logic.
- **`mana_ability` vs `activated_ability` disjointness** (SR-34/SF-6) must be preserved in the shared
  helper — an ability lowered to `mana_abilities` must not also appear in `activated_abilities`.
- **Mutate + transform interaction** — `layers.rs:246` takes the top merged component's characteristics
  (including its ability vectors) for a mutated permanent; a mutated *and* transformed DFC is an
  untested corner. Out of roster scope; note as a known limitation, do not regress the merged path.
- **Copy of a transformed permanent** (CR 707.2/712.8e) — copy uses copiable values (front unless
  copying the back), handled in `copy.rs`, not via `is_transformed` ability gathering. Confirm this PB
  does not perturb it (front-only, no change).
- **Cost/perf** — `apply_face_change` runs only at transform boundaries (rare); `build_face_ability_vectors`
  is O(back-face abilities). No hot-path change (`calculate_characteristics` untouched for ability
  vectors).
