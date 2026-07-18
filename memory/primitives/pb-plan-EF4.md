# Primitive Batch Plan: PB-EF4 — TriggeringCreature as effect subject/source

**Generated**: 2026-07-18
**Primitive**: Two capability additions in Cluster B ("the just-triggered object as the
effect's subject/source"), building on PB-EF3's `triggering_creature_id` threading:
  - **(1) EF-W-PB2-6 ≡ EF-W-MISS-5** — add `EffectFilter::TriggeringCreature`, a
    continuous-effect filter (`crates/card-types/src/state/continuous_effect.rs`) that
    resolves to `SingleObject(ctx.triggering_creature_id)` at `Effect::ApplyContinuousEffect`
    execution time (mirroring `EffectFilter::Source`/`DeclaredTarget`). Lets "when a creature
    enters/attacks, **it** gains \<keyword\> / gets +N/+N until end of turn" be expressed.
  - **(2) EF-W-PB2-7** — add `source: Option<EffectTarget>` to `Effect::DealDamage`. `None`
    = existing behaviour (damage source = `ctx.source`); `Some(t)` resolves `t` to a single
    ObjectId and uses THAT as the damage source for doubling, prevention, infect/lifelink/
    deathtouch keyword reads (CR 702.15/702.2/702.90), lifelink-gain controller, and the
    `source:` field of `DamageDealt`/`PoisonCountersGiven` events. `source:
    Some(EffectTarget::TriggeringCreature)` = "the entering/attacking creature deals it."
**CR Rules**: 119 (damage), 702.15 (lifelink), 702.2 (deathtouch), 702.90 (infect),
613.1f/611.2 (continuous effects / Layer 6), 603.2/603.6a (triggered abilities),
508.1m (attacks trigger), 400.7/113.7a (object identity / LKI).
**Cards affected**: **7 ship** (2 flip-inert, 3 flip-partial, 2 new) + 3 blocked/out-of-scope.
**Dependencies**: none new. All prerequisites exist:
`EffectContext.triggering_creature_id` (PB-EF3, threaded StackObject→EffectContext at
resolution.rs:2109/2202, set at abilities.rs:7930 from `PendingTrigger.entering_object_id`),
`EffectTarget::TriggeringCreature` (effects/mod.rs:6330, resolves to a single Object),
`WheneverCreatureEntersBattlefield { filter, exclude_self }` /
`WheneverPermanentEntersBattlefield { filter, exclude_self }` /
`WheneverCreatureYouControlAttacks { filter }` trigger conditions,
`EffectAmount::PowerOf(EffectTarget::TriggeringCreature)`, `PermanentCount`, `Amass`.
**Deferred items from prior PBs**: this PB IS the scheduled owner of EF-W-PB2-6 (≡ EF-W-MISS-5)
and EF-W-PB2-7 (Cluster B in `ef-batch-plan-2026-07-17.md` §1a). No other carry-forward.

---

## TODO sweep (roster-recall gate — MANDATORY, per feedback_planner_roster_recall)

Grepped `crates/card-defs/src/defs/` for pre-existing blocker notes naming the two PB-EF4
primitives. **Result: 2 forced adds beyond the 8-card brief roster.**

**`EffectFilter::TriggeringCreature` (continuous) self-identifiers** (`TriggeringCreature`
in a completeness/blocker note, not oracle text): `dragon_tempest.rs`, `ogre_battledriver.rs`,
`shared_animosity.rs` (all in brief) **plus `dreadhorde_invasion.rs`** — its `partial` marker
reads verbatim: *"The gap is EffectFilter, which has no TriggeringCreature variant, so the
until-EOT lifelink grant cannot be aimed at the attacker that fired the trigger."* **FORCED
ADD — not in the PB brief.**

**`Effect::DealDamage` source-override self-identifiers** (`sources from ctx.source` /
"always uses the ability's source" damage-attribution notes): `scourge_of_valkas.rs`,
`dragon_tempest.rs` (both in brief) **plus `warstorm_surge.rs`** — its `partial` marker reads:
*"Blocked on damage source attribution: oracle is 'it deals damage equal to its power', so the
entering creature must be the damage source … but Effect::DealDamage always uses the ability's
source. The trigger and PowerOf(TriggeringCreature) amount ARE wired and verified."* **FORCED
ADD — not in the PB brief.**

I then swept **every** `it deals damage` note in the corpus (grep of oracle/comment text) to
confirm no third DealDamage-source card was missed. Of ~20 hits, only `warstorm_surge` has
"it" = a creature *other than* `ctx.source`. All others are self-referential ("this creature
enters/dies/attacks, it deals …" where the source IS the ability's own permanent —
omnath_locus_of_the_roil, juri, volley_veteran, goblin_chainwhirler, balefire_dragon,
spiteful_banditry, drakuseth, brallin, legolas×2, terror_of_the_peaks) and correctly source
from `ctx.source` — **explicitly NOT PB-EF4 cards**. (Terror of the Peaks is the deliberate
contrast case: "**this creature** deals damage equal to that creature's power," source =
Terror = ctx.source, no override — do not touch it.)

**Net roster: 7 ship** (brief's 5 shippers + 2 forced adds). This is exactly the yield-recall
win the gate exists to catch — the brief estimated ~4–5.

---

## Primitive Specification

### Part 1 — `EffectFilter::TriggeringCreature` (continuous-effect subject)

`EffectFilter` (`continuous_effect.rs:67`, the filter on `ContinuousEffectDef`) has
`Source`/`DeclaredTarget`/`CreaturesYouControl`/… but **no `TriggeringCreature`**. Only the
point-effect `EffectTarget` enum has it. So a continuous grant aimed at "the creature that just
triggered this" (haste/double-strike/lifelink/+N/+N until end of turn on the entering-or-
attacking creature) is inexpressible. The new variant mirrors `Source` exactly: it is a
runtime placeholder resolved at `Effect::ApplyContinuousEffect` execution to
`SingleObject(ctx.triggering_creature_id)`; if `triggering_creature_id` is `None`, the effect
applies to nothing (skip, no panic).

**Why it works for both ETB and attack triggers** (chain-verified): `triggering_creature_id`
is populated from `PendingTrigger.entering_object_id` at `flush_pending_triggers`
(abilities.rs:7930), and `entering_object_id` is set to the entering permanent for ETB
triggers **and** to the attacker for `AnyCreatureYouControlAttacks` triggers
(abilities.rs:3914–3922 passes `Some(*attacker_id)` → collect sets `entering_object_id`,
abilities.rs:6402). Threaded to `ctx.triggering_creature_id` at resolution.rs:2109/2202 (PB-EF3).
The `ApplyContinuousEffect` executor reads `ctx` at exactly the substitution site — so the
placeholder resolves for every ETB/attack trigger this PB targets.

### Part 2 — `Effect::DealDamage { source: Option<EffectTarget> }`

`DealDamage { target, amount }` (`card_definition.rs:1330`) always sources from `ctx.source`.
So "when another permanent enters, **it** deals X damage" (the entering permanent as the
damage source) is inexpressible — Dragon Tempest is never itself a Dragon, so it misattributes
on 100% of firings. Adding `source: Option<EffectTarget>` (`None` = `ctx.source`, unchanged;
`Some(t)` = resolve `t` to one ObjectId) lets the damage be sourced from the triggering
creature, so infect/lifelink/deathtouch/protection/redirection and event attribution all read
the correct source's characteristics (CR 702.15a lifelink, 702.2b deathtouch, 702.90 infect).

---

## CR Rule Text (key excerpts)

- **119.3** "The source of the damage is the object that dealt it… A source's controller… is
  determined … as though it were on the stack" — the damage-source object drives lifelink/
  infect/deathtouch and event attribution. Justifies threading a single `damage_source_id`.
- **702.15a** "Lifelink is a static ability. Damage dealt by a source with lifelink also causes
  its controller to gain that much life." — lifelink reads the **source's** keyword and gains
  for the **source's controller**, not the ability's controller. This is why `source:
  Some(TriggeringCreature)` must retarget `damage_source_characteristics` AND
  `damage_source_controller`, not just the event's `source:` field.
- **702.2b** "A creature that's been dealt damage by a source with deathtouch since the last
  time state-based actions were checked is destroyed…" — deathtouch is read off the source.
- **702.90b/e** infect functions from any zone and reads off the source.
- **611.2a/613.1f** a continuous effect from a resolving spell/ability locks in its affected
  object at resolution; `EffectFilter::TriggeringCreature` → `SingleObject(id)` at execution is
  exactly this "lock in the object now" behaviour (identical to how `Source`/`DeclaredTarget`
  are substituted before the `ContinuousEffect` is stored).
- **603.6a/508.1m** ETB and attack triggered abilities; the triggering object is fixed at
  trigger time (LKI, CR 113.7a) — matches the capture already done by PB-EF3.

---

## Engine Changes (ordered)

### Change 1 — add `EffectFilter::TriggeringCreature` variant

**File**: `crates/card-types/src/state/continuous_effect.rs`
**Action**: add a variant to the `EffectFilter` enum (after `Source` at ~L119, keep it grouped
with the other runtime-placeholder variants):
```rust
/// Applies to the creature that triggered the ability creating this effect
/// (the entering or attacking creature). Resolved at `ApplyContinuousEffect`
/// execution time to `SingleObject(ctx.triggering_creature_id)` — mirrors
/// `Source`. If `triggering_creature_id` is `None`, the effect applies to
/// nothing. Used by "when a creature enters/attacks, IT gains <keyword> /
/// gets +N/+N until end of turn" (Dragon Tempest, Ogre Battledriver, Atarka,
/// Fervent Charge, Dreadhorde Invasion). CR 611.2a / 613.1f.
TriggeringCreature,
```
**Pattern**: `EffectFilter::Source` (the closest sibling — same "runtime placeholder,
substituted before storage" contract).

### Change 2 — resolve the placeholder at the `ApplyContinuousEffect` executor

**File**: `crates/engine/src/effects/mod.rs` (the `resolved_filter` match at L3004–3030 inside
the `Effect::ApplyContinuousEffect` arm)
**Action**: add an arm alongside `CEFilter::Source => CEFilter::SingleObject(ctx.source)`:
```rust
// CR 611.2a: the triggering creature (entering/attacking) as the effect's subject.
CEFilter::TriggeringCreature => match ctx.triggering_creature_id {
    Some(id) => CEFilter::SingleObject(id),
    None => return, // no triggering creature captured — apply to nothing (no panic).
},
```
This match has an `other => other.clone()` fallback, so the arm is *semantically* required
(not compile-forced) — without it, `TriggeringCreature` would be stored raw and then match
nothing at layer time (see Change 3), silently no-op'ing. Add the explicit arm.
**CR**: 611.2a / 613.1f.

### Change 3 — static-registration matcher must recognize the new variant

**File**: `crates/engine/src/rules/layers.rs` (`matches_filter`, the exhaustive `match` on
`EffectFilter` — `Source => false` at L653, `DeclaredTarget { .. } => false` at L650)
**Action**: add `EffectFilter::TriggeringCreature => false,` beside them. This match is
**exhaustive (no wildcard)** → compile error until added. A stored `TriggeringCreature` (which
should never happen — it is always substituted to `SingleObject` in Change 2) matches nothing,
same as `Source`.

**File**: `crates/engine/src/rules/replacement.rs` (static-ability registration, `resolved_filter`
match at L2057) — **no change needed**: it uses `other => other.clone()` and
`TriggeringCreature` never appears on a *static* ability's continuous effect (it is a one-shot
spell/triggered-ability effect only). Note in the plan for completeness; do not add an arm.

### Change 4 — hash the new `EffectFilter` discriminant

**File**: `crates/engine/src/state/hash.rs` (`impl HashInto for EffectFilter`, exhaustive match
ending at discriminant 34 / `CreaturesYouControlOfChosenColor` at L1921)
**Action**: add `EffectFilter::TriggeringCreature => 35u8.hash_into(hasher),` (next free
discriminant byte). Exhaustive match → compile-forced.

### Change 5 — add `source` field to `Effect::DealDamage`

**File**: `crates/card-types/src/cards/card_definition.rs:1330`
**Action**: reshape the variant:
```rust
/// CR 119: Deal damage to a target (player, creature, or planeswalker).
DealDamage {
    target: EffectTarget,
    amount: EffectAmount,
    /// CR 119.3 / 702.15a: the damage source. `None` = the ability's source
    /// (`ctx.source`, existing behaviour). `Some(t)` resolves `t` to a single
    /// ObjectId used as the damage source everywhere in the executor — doubling,
    /// prevention, infect/lifelink/deathtouch keyword reads, lifelink-gain
    /// controller, and the `source:` of DamageDealt/PoisonCountersGiven. Used by
    /// "when a creature enters, IT deals damage" (Warstorm Surge, Dragon Tempest,
    /// Scourge of Valkas) with `Some(EffectTarget::TriggeringCreature)`.
    #[serde(default)]
    source: Option<EffectTarget>,
},
```
**`#[serde(default)]` decision — YES, add it.** Justification: (a) it matches the pervasive
codebase convention — every added `Effect`/DSL field in `card_definition.rs` carries
`#[serde(default)]` (30+ instances, incl. the `Effect` enum's own fields); (b) `Option`'s
Default is `None` = the exact "unchanged behaviour" value; (c) it costs nothing. It does **NOT**
reduce the construction-site blast radius (that is unavoidable — see Change 8). Note this
differs from PB-EF2's `TokenSpec.recipient`, where `#[serde(default)]` *did* keep 201 sites
compiling **because** `TokenSpec` has a `Default` impl and card defs use `..Default::default()`;
`Effect::DealDamage` is an enum struct-variant with no `Default` and no `..` rest syntax, so
every literal must be touched regardless. Strict-lockstep (SR-8) makes wire tolerance moot, but
convention-consistency is the deciding factor.

### Change 6 — thread a single `damage_source_id` through the executor

**File**: `crates/engine/src/effects/mod.rs`, the `Effect::DealDamage` arm (L271–512)
**Action**:
1. Update the match pattern: `Effect::DealDamage { target, amount, source } => {`.
2. At the top of the arm (before the `resolve_effect_target_list(state, target, ctx)` on the
   *damage target*), compute the source id once:
```rust
// CR 119.3: resolve the damage source. Default = ctx.source (source: None).
// Some(t): use the first resolved Object as the source; if it resolves to no
// object (e.g. TriggeringCreature already left, CR 113.7a), fall back to ctx.source.
let damage_source_id = source
    .as_ref()
    .and_then(|t| {
        resolve_effect_target_list(state, t, ctx)
            .into_iter()
            .find_map(|r| match r {
                ResolvedTarget::Object(id) => Some(id),
                ResolvedTarget::Player(_) => None,
            })
    })
    .unwrap_or(ctx.source);
```
3. Replace **every** `ctx.source` read in this arm with `damage_source_id`. Exhaustive list
   (both the Player branch L280–373 and the Object branch L375–508):
   - L289 `apply_damage_doubling(state, ctx.source, …)` (Player)
   - L300 `apply_damage_prevention(state, ctx.source, …)` (Player)
   - L310 `damage_source_characteristics(state, ctx.source)` (Player infect/lifelink)
   - L329, L336 `source: ctx.source` (`DamageDealt`, `PoisonCountersGiven`, infect Player path)
   - L349 `source: ctx.source` (`DamageDealt`, non-infect Player path)
   - L361 `damage_source_controller(state, ctx.source)` (Player lifelink gain)
   - L391 `apply_damage_doubling(state, ctx.source, …)` (Object)
   - L406 `apply_damage_prevention(state, ctx.source, …)` (Object)
   - L416 `damage_source_characteristics(state, ctx.source)` (Object wither/infect/deathtouch/lifelink)
   - L491 `damage_source_controller(state, ctx.source)` (Object lifelink gain)
   - L504 `source: ctx.source` (`DamageDealt`, Object path)
   **Do not** rename the loop over the damage *targets* — `target` still resolves the
   recipients; only the source moves. Leave any *other* `ctx.*` reads (e.g. `ctx.controller`)
   untouched.
**CR**: 119.3 / 702.15a / 702.2b / 702.90b.

### Change 7 — hash the new `source` field on `Effect::DealDamage`

**File**: `crates/engine/src/state/hash.rs:5762` (`impl HashInto for Effect`, `DealDamage` arm)
**Action**: bind and hash the field:
```rust
Effect::DealDamage { target, amount, source } => {
    0u8.hash_into(hasher);
    target.hash_into(hasher);
    amount.hash_into(hasher);
    source.hash_into(hasher);   // Option<EffectTarget>: HashInto already impl'd
}
```
Exhaustive match → compile-forced. (`Option<T: HashInto>` and `EffectTarget` both already
impl `HashInto`; confirm — `EffectTarget` is hashed elsewhere.)

### Change 8 — `Effect::DealDamage` construction-site migration (~115 sites)

Because `DealDamage` is an enum struct-variant with no `Default`/`..`, **every literal must add
`source: None,`** (field order is free, so no per-site reasoning is needed except the 3 override
cards below). Counts: **114 occurrences across 90 files** in `crates/card-defs/src/defs/` +
**1 in `crates/engine/src/testing/replay_harness.rs:3888`** (the pain-land pattern) + any in
`card-types` the compiler surfaces.

**Safest mechanical approach**:
1. Bulk-insert `source: None,` immediately after each `Effect::DealDamage {` opener across the
   corpus (single, unambiguous anchor; field order irrelevant), e.g.
   `rg -l 'Effect::DealDamage \{' crates/card-defs/src/defs | xargs sed -i \
   's/Effect::DealDamage {/Effect::DealDamage {\n            source: None,/'` — indentation
   will be non-canonical but **`tools/check-defs-fmt.sh --fix` normalizes it** afterward.
2. Apply the same to `replay_harness.rs:3888` (hand-edit; it's a single site) — this file is
   engine code, so `cargo fmt` covers it (not `check-defs-fmt.sh`).
3. For the **3 source-override cards** (scourge_of_valkas, warstorm_surge, dragon_tempest's
   Dragon half), set `source: Some(EffectTarget::TriggeringCreature)` instead of `None` — do
   these by hand as part of the card fixes (Change/Card section below), not in the bulk sed.
4. `cargo build --workspace` is the backstop: it names **every** remaining unmigrated literal
   (including any exhaustive `match` on `Effect` in `tools/` that binds `DealDamage`). Do not
   trust the 115 count — let the compiler close the set.
5. `tools/check-defs-fmt.sh` (or `cargo test --all` via `core card_defs_fmt`) must stay green;
   run `--fix` then verify no `error_on_line_overflow` (the added short line cannot overflow).

### Change 9 — exhaustive-match / wire updates (summary)

| File | Match / gate | Action |
|------|--------------|--------|
| `crates/card-types/src/state/continuous_effect.rs` | `EffectFilter` enum | add `TriggeringCreature` variant (Change 1) |
| `crates/engine/src/rules/layers.rs` | `matches_filter` (exhaustive `EffectFilter`) | `TriggeringCreature => false` (Change 3) — **compile-forced** |
| `crates/engine/src/state/hash.rs` | `HashInto for EffectFilter` (exhaustive) | `TriggeringCreature => 35u8` (Change 4) — **compile-forced** |
| `crates/engine/src/effects/mod.rs` | `ApplyContinuousEffect` `resolved_filter` (has `other`) | `TriggeringCreature => SingleObject` arm (Change 2) — semantic, add it |
| `crates/card-types/src/cards/card_definition.rs` | `Effect::DealDamage` variant | add `source` field (Change 5) |
| `crates/engine/src/effects/mod.rs` | `Effect::DealDamage` executor arm | bind `source`, thread `damage_source_id` (Change 6) |
| `crates/engine/src/state/hash.rs` | `HashInto for Effect` (exhaustive) | bind + hash `source` (Change 7) — **compile-forced** |
| ~115 construction sites | `Effect::DealDamage { … }` literals | add `source: None,` (Change 8) — **compile-forced** |
| `crates/engine/src/rules/replacement.rs` | `resolved_filter` (has `other`) | **none** — `other.clone()` handles it; documented |
| `crates/engine/src/rules/copy.rs` | `match &effect.filter` (has `_ => false`) | **none** — wildcard covers it |
| `crates/engine/src/rules/protocol.rs` | `PROTOCOL_VERSION` | **8 → 9** (see wire) |
| `crates/engine/src/state/hash.rs` | `HASH_SCHEMA_VERSION` | **46 → 47** (see wire) |
| — | any *other* exhaustive `match` on `EffectFilter` or `Effect::DealDamage` | **run `cargo build --workspace`**; the compiler is the authority — this table is not assumed exhaustive |

### Wire-bump procedure (machine-forced — do not guess numbers)

Both changes reach the SR-8 fingerprint closure (`Effect` → the card DSL → the three wire
frames) and the `GameState` hash closure (`Characteristics.abilities` → `Effect`,
`ContinuousEffect.filter` → `EffectFilter`).

- **PROTOCOL 8 → 9**: `tests/protocol_schema.rs` recomputes `PROTOCOL_SCHEMA_FINGERPRINT` from
  source and fails naming the drift (new `EffectFilter` variant + reshaped `DealDamage`). Bump
  `PROTOCOL_VERSION` to 9, re-pin `PROTOCOL_SCHEMA_FINGERPRINT` to the value the failing test
  prints, and append a `- 9: PB-EF4 (2026-07-18) — …` `# History` row to the doc-comment in
  `rules/protocol.rs` (the closure type-count is unchanged: `EffectTarget` and `EffectFilter`
  are already in the closure; only their declared shapes move).
- **HASH 46 → 47**: `tests/core/hash_schema.rs` recomputes `decl_fingerprint` (serialized shape)
  and `stream_fingerprint` (HashInto byte stream) and forces the bump. Set
  `HASH_SCHEMA_VERSION = 47`, append a `- 47: PB-EF4 …` line to the `History:` block, and re-pin
  both digests in `HASH_SCHEMA_HISTORY` to the printed values.
- **Let the gates drive both.** Do not pre-edit the constants; make the code changes, run the
  suites, and re-pin from the failure output (SR-8/SR-17 convention).

---

## Card Definition Fixes / New Cards

### SHIP (7)

#### `dragon_tempest.rs` — FLIP `inert` → **Complete** (BOTH primitives)
**Oracle**: "Whenever a creature you control with flying enters, it gains haste until end of
turn. / Whenever a Dragon you control enters, it deals X damage to any target, where X is the
number of Dragons you control." {1}{R} Enchantment.
**Fix** (replace `abilities: vec![]` + drop the `inert` marker):
- Ability A (flying half): `AbilityDefinition::Triggered { trigger_condition:
  WheneverCreatureEntersBattlefield { filter: Some(TargetFilter { has_keywords: {Flying},
  controller: You, .. }), exclude_self: false }, effect: ApplyContinuousEffect {
  ContinuousEffectDef { layer: Ability, modification: AddKeyword(Haste), filter:
  EffectFilter::TriggeringCreature, duration: UntilEndOfTurn, condition: None } }, targets:
  vec![], .. }`.
- Ability B (Dragon half): `Triggered { trigger_condition: WheneverCreatureEntersBattlefield {
  filter: Some(TargetFilter { has_subtype: Dragon, controller: You, .. }), exclude_self: false
  }, effect: DealDamage { target: DeclaredTarget{0}, amount: PermanentCount { filter: {
  has_subtype: Dragon }, controller: Controller }, source: Some(EffectTarget::TriggeringCreature)
  }, targets: vec![TargetRequirement::TargetAny], .. }`.
**Chain**: X counts all Dragons you control **including** the just-entered one (it is now on the
battlefield) → `PermanentCount` with **no `exclude_self`**, correct per oracle. Source =
entering Dragon → `Some(TriggeringCreature)`. **Runner-verify**: `TargetFilter.has_keywords`
is honored by the ETB filter path in `collect_triggers_for_event`/`matches_filter` (it is a
standard filter field; confirm the flying-ETB filter actually matches).

#### `scourge_of_valkas.rs` — FLIP `partial` → **Complete** (DealDamage source)
**Oracle**: "Flying / Whenever this creature or another Dragon you control enters, it deals X
damage to any target, where X is the number of Dragons you control. / {R}: This creature gets
+1/+0 until end of turn." 4/4 {2}{R}{R}{R} Creature — Dragon.
**Fix**: the def currently authors only the **self**-ETB half (`WhenEntersBattlefield`) because
"another Dragon" couldn't be sourced. Merge both halves into ONE trigger (Scourge is itself a
Dragon, so "this creature or another Dragon you control" = "a Dragon you control"):
- Change the trigger from `WhenEntersBattlefield` to `WheneverCreatureEntersBattlefield {
  filter: Some(TargetFilter { has_subtype: Dragon, controller: You, .. }), exclude_self: false
  }` (`exclude_self: false` = includes Scourge itself).
- Add `source: Some(EffectTarget::TriggeringCreature)` to the existing `DealDamage`. When
  Scourge enters, `triggering_creature_id` = Scourge (= ctx.source, so identical to old
  behaviour); when another Dragon enters, it = that Dragon. Amount/target unchanged.
- Keep the `{R}: +1/+0` activated ability (already correct via `EffectFilter::Source`).
- Drop the `partial(...)` marker.

#### `ogre_battledriver.rs` — FLIP `inert` → **Complete** (TriggeringCreature filter, ×2 mods)
**Oracle**: "Whenever another creature you control enters, that creature gets +2/+0 and gains
haste until end of turn." 3/3 {2}{R}{R} Creature — Ogre Warrior.
**Fix** (replace `abilities: vec![]` + drop `inert`): one `Triggered { trigger_condition:
WheneverCreatureEntersBattlefield { filter: Some(TargetFilter { controller: You, .. }),
exclude_self: true }, effect: Sequence[ ApplyContinuousEffect { ContinuousEffectDef { layer:
PtModify, modification: ModifyPower(2), filter: TriggeringCreature, duration: UntilEndOfTurn }},
ApplyContinuousEffect { ContinuousEffectDef { layer: Ability, modification: AddKeyword(Haste),
filter: TriggeringCreature, duration: UntilEndOfTurn }} ], targets: vec![], .. }`.
**Chain**: `exclude_self: true` = "another" (Ogre's own ETB does not fire). Two continuous
effects, both aimed at the entering creature via `TriggeringCreature`. The def's existing
comment already names this exact shape.

#### `atarka_world_render.rs` — NEW, author **Complete** (TriggeringCreature filter)
**Oracle**: "Flying, trample / Whenever a Dragon you control attacks, it gains double strike
until end of turn." 6/4 {5}{R}{G} Legendary Creature — Dragon.
**Sketch**: keywords Flying + Trample; `Triggered { trigger_condition:
WheneverCreatureYouControlAttacks { filter: Some(TargetFilter { has_subtype: Dragon, .. }) },
effect: ApplyContinuousEffect { ContinuousEffectDef { layer: Ability, modification:
AddKeyword(DoubleStrike), filter: TriggeringCreature, duration: UntilEndOfTurn }}, targets:
vec![], .. }`. Legendary supertype. (File does not exist — author fresh.)

#### `fervent_charge.rs` — NEW, author **Complete** (TriggeringCreature filter)
**Oracle**: "Whenever a creature you control attacks, it gets +2/+2 until end of turn."
{1}{R}{W}{B} Enchantment.
**Sketch**: `Triggered { trigger_condition: WheneverCreatureYouControlAttacks { filter: None },
effect: ApplyContinuousEffect { ContinuousEffectDef { layer: PtModify, modification:
ModifyBoth(2), filter: TriggeringCreature, duration: UntilEndOfTurn }}, targets: vec![], .. }`.
(File does not exist — author fresh.)

#### `dreadhorde_invasion.rs` — FLIP `partial` → **Complete** (TODO-sweep forced add)
**Oracle**: "At the beginning of your upkeep, you lose 1 life and amass Zombies 1. … / Whenever
a Zombie token you control with power 6 or greater attacks, it gains lifelink until end of
turn." {1}{B} Enchantment.
**Current**: upkeep half authored Complete (LoseLife + Amass); attack half omitted, `partial`
marker names the `EffectFilter::TriggeringCreature` gap **and** gives the exact trigger shape.
**Fix**: add `Triggered { trigger_condition: WheneverCreatureYouControlAttacks { filter:
Some(TargetFilter { has_subtype: Zombie, min_power: Some(6), is_token: true, .. }) }, effect:
ApplyContinuousEffect { ContinuousEffectDef { layer: Ability, modification:
AddKeyword(Lifelink), filter: TriggeringCreature, duration: UntilEndOfTurn }}, targets:
vec![], .. }`; drop the `partial(...)` marker. **Runner-verify**: `TargetFilter.min_power` +
`is_token` are honored on the attack-trigger filter path (the def's marker asserts PB-N wired
this at abilities.rs:6092/6108 — confirm by test). *Added via pre-existing TODO sweep — not in
original PB brief.*

#### `warstorm_surge.rs` — FLIP `partial` → **Complete** (TODO-sweep forced add)
**Oracle**: "Whenever a creature you control enters, it deals damage equal to its power to any
target." {5}{R} Enchantment.
**Current**: trigger + `DealDamage { target: DeclaredTarget{0}, amount:
PowerOf(TriggeringCreature) }` already wired and verified; `partial` marker names the missing
source-override as the sole blocker.
**Fix**: add `source: Some(EffectTarget::TriggeringCreature)` to the existing `DealDamage`; drop
the `partial(...)` marker. The entering creature is now both the amount reference AND the damage
source, so its lifelink/deathtouch/protection interactions are correct. *Added via pre-existing
TODO sweep — not in original PB brief.*

### BLOCKED / OUT OF SCOPE (3 — do NOT author; no gated-stub effects)

- **`shared_animosity.rs` — BLOCKED (stays `inert`; file OOS-EF4-1)**. "Whenever a creature you
  control attacks, it gets +1/+0 until end of turn **for each other attacking creature that
  shares a creature type with it**." PB-EF4 supplies the *subject* (`EffectFilter::
  TriggeringCreature`), but the **amount** — a per-trigger count of *other attacking creatures
  sharing a creature type with the triggering creature* — has no `EffectAmount` variant (it
  needs a dynamic count keyed on the trigger source's layer-resolved subtypes vs. every other
  attacker's subtypes). The def's own note documents both gaps; only one is closed here. **File
  OOS-EF4-1** (per-trigger "attacking creatures matching a property of the triggering creature"
  count `EffectAmount`).
- **`goblin_piledriver.rs` — OUT OF SCOPE (do not create)**. "Protection from blue / Whenever
  **this creature** attacks, it gets +2/+0 for each other attacking Goblin." Subject is
  `ctx.source` (self-attack — `EffectFilter::Source`, not `TriggeringCreature`), so PB-EF4 does
  not unblock it. Blockers are (a) an "other attacking Goblins" count `EffectAmount` and (b)
  protection-from-blue as a filtered protection. Different primitives.
- **`muxus_goblin_grandee.rs` — OUT OF SCOPE (do not create)**. ETB reveals top six and puts
  Goblin creatures MV≤5 onto the battlefield (a reveal + conditional-put + bottom-random
  effect — no primitive), and the attack half is a self-attack ("Muxus … it gets +1/+1 for each
  other Goblin you control", subject = `ctx.source`). Neither PB-EF4 primitive is Muxus's
  blocker; the ETB alone keeps it non-Complete.

**Contrast (do NOT touch)**: `terror_of_the_peaks.rs` — "**this creature** deals damage equal to
that creature's power" — source = Terror = `ctx.source`, correct with `source: None`. It is the
deliberate negative case for the source-override and must keep `source: None` (just the bulk
migration, no `Some`).

---

## Unit Tests

**File**: `crates/engine/tests/primitives/pb_ef4_triggering_creature_subject_source.rs` (new).
**Register**: add `mod pb_ef4_triggering_creature_subject_source;` to
`crates/engine/tests/primitives/main.rs` (after `pb_ef3b_granted_keyword_triggers`).
**Patterns**: ETB/attack setup + `enrich_spec_from_def` inline specs from
`pb_ef3_attack_trigger_targets.rs`; continuous-grant + P/T assertion via
`calculate_characteristics` from `pb_ef3b_granted_keyword_triggers.rs`; damage-source/LKI
assertions from `primitives/sr13_lki_damage_source.rs`. Every decoy must be provably
non-vacuous (revert the fix → it reddens).

### REQUIRED decoy tests (2)

1. `test_ef4_triggering_creature_filter_selects_exactly_the_trigger_source` — **CR 611.2a**.
   Ogre-Battledriver-style ETB grant (`ModifyPower(2)` + `AddKeyword(Haste)` UEOT via
   `EffectFilter::TriggeringCreature`). Put a **same-type decoy** creature on the battlefield
   *before* the ETB; a second creature then enters. Assert: the **entering** creature has power
   +2 and haste; the **decoy** is unchanged (base P/T, no haste). **Non-vacuity**: swap the
   filter to `EffectFilter::CreaturesYouControl` → the decoy also gets pumped/hasted → test
   reddens. (Guards against "applies to all my creatures" and "applies to source".)

2. `test_ef4_dealdamage_source_override_attributes_lifelink_to_source_controller` — **CR
   702.15a**. Inline `DealDamage { target: Player(opp), amount: Fixed(3), source:
   Some(TriggeringCreature) }` where the triggering creature **has Lifelink** and is controlled
   by player **A**, while the ability's `ctx.source`/controller is a different player **B**
   (e.g. a shared enchantment, or A's creature triggering B's ability — construct so
   source-controller ≠ ability-controller). Assert: **A** (the damage source's controller) gains
   3 life; **B** does not. **Companion regression decoy**
   `test_ef4_dealdamage_source_none_default_path_unchanged` — the same `DealDamage` with
   `source: None` and a **non-lifelink** `ctx.source`: assert **no** life gain (damage read off
   `ctx.source`, unchanged). **Non-vacuity**: revert Change 6 (thread `damage_source_id`) so
   lifelink reads `ctx.source` → the override test reddens (B's non-lifelink source → A gains
   nothing).

### Per-card trigger-level tests (7)

- `test_ef4_dragon_tempest_flying_grants_haste_dragon_deals_damage` — a flyer enters → gains
  haste UEOT (a non-flyer enterer does not); a Dragon enters → deals `X` = Dragon count to a
  chosen target, and `GameEvent::DamageDealt.source == the entering Dragon's id` (not the
  enchantment). CR 603.6a / 119.3.
- `test_ef4_scourge_of_valkas_self_and_another_dragon` — Scourge's own ETB deals damage (source
  = Scourge); a *second* Dragon entering also deals damage with `source == that Dragon`. One
  trigger, both halves. CR 508/119.3.
- `test_ef4_ogre_battledriver_pumps_and_hastes_only_the_enterer` — another creature enters →
  +2/+0 and haste; a decoy already-present creature unchanged; **Ogre's own ETB does not fire**
  (`exclude_self: true`). CR 603.6a.
- `test_ef4_atarka_grants_double_strike_to_attacking_dragon` — a Dragon attacks → gains double
  strike UEOT; a non-Dragon attacker does not (filter). CR 508.1m.
- `test_ef4_fervent_charge_pumps_attacking_creature` — a creature attacks → +2/+2 UEOT; a
  non-attacking creature you control is unbuffed. CR 508.1m.
- `test_ef4_dreadhorde_invasion_lifelink_gated_by_token_and_power` — a Zombie **token** with
  power ≥6 attacks → gains lifelink UEOT; a power-5 Zombie token **or** a non-token Zombie does
  NOT (filter `min_power`/`is_token`). CR 508.1m / 702.15a.
- `test_ef4_warstorm_surge_entering_creature_deals_its_power_from_itself` — a creature enters →
  deals damage = its power to a chosen target, `DamageDealt.source == the entering creature`
  (not Warstorm Surge). Add a lifelink-enterer variant asserting the enterer's controller gains
  life (proves the source override drives lifelink, not just the event field). CR 119.3/702.15a.

**Regression**: run `primitives/sr13_lki_damage_source.rs`, `combat/`, and any existing
`DealDamage` suites unchanged — they exercise the `source: None` default path and must stay
green (proves Change 6 is behaviour-preserving for the 112 unmigrated-intent sites).

---

## Verification Checklist

- [ ] `EffectFilter::TriggeringCreature` added + resolved at `ApplyContinuousEffect`
      (SingleObject / None→skip) + `matches_filter => false` + hash disc 35
- [ ] `Effect::DealDamage.source: Option<EffectTarget>` added with `#[serde(default)]`;
      executor threads one `damage_source_id` through all 11 `ctx.source` reads; hashed
- [ ] All ~115 `DealDamage` construction sites carry `source: None,` (3 override cards carry
      `Some(TriggeringCreature)`); `cargo build --workspace` clean (backstop for missed sites)
- [ ] `cargo check -p mtg-engine` then `cargo build --workspace` (GameState seal + exhaustive
      `EffectFilter`/`Effect` match gate)
- [ ] `PROTOCOL_VERSION` 8→9 + fingerprint re-pin + history row; `HASH_SCHEMA_VERSION` 46→47 +
      both digest re-pins + history row (driven by the failing schema/hash gates, not guessed)
- [ ] 7 cards Complete: dragon_tempest, scourge_of_valkas, ogre_battledriver (flips) +
      atarka_world_render, fervent_charge (new) + dreadhorde_invasion, warstorm_surge (flips);
      no remaining TODO/partial markers on them
- [ ] shared_animosity stays `inert` (OOS-EF4-1 filed); goblin_piledriver + muxus NOT created;
      terror_of_the_peaks kept `source: None`
- [ ] New `pb_ef4_*` tests pass incl. both required decoys (non-vacuity verified by revert);
      `sr13_lki_damage_source` + combat suites green (source:None regression)
- [ ] `cargo test --all`; `cargo clippy --all-targets -- -D warnings`;
      `cargo fmt --check` **and** `tools/check-defs-fmt.sh` clean

---

## Risks & Edge Cases

- **The 115-site migration is the primary risk.** Field order is free, so a uniform
  `source: None,` insert is safe, but a missed site is a hard compile error (good) — never a
  silent wrong-state. `cargo build --workspace` is the authoritative backstop; do not rely on
  the grep count. Keep `check-defs-fmt.sh` green after the bulk edit (`--fix` normalizes the
  sed's indentation; verify no line-overflow).
- **`damage_source_id` LKI fallback** (CR 113.7a): if `source: Some(TriggeringCreature)` and the
  triggering creature has already left before the damage resolves, `resolve_effect_target_list`
  returns no Object and we fall back to `ctx.source`. Acceptable degradation (the entering/
  attacking creature is present at resolution in every realistic case), and it matches how the
  rest of the engine degrades. Note in the executor comment; do NOT panic.
- **`source` resolving to a Player** (nonsensical author input like `Some(Controller)`): the
  `find_map` takes only the first `Object`; a Player-only resolution falls back to `ctx.source`.
  Harmless.
- **`TargetFilter` field honoring** (chain-verify at impl): dragon_tempest's flying half depends
  on `has_keywords` being matched by the ETB filter path; dreadhorde depends on `min_power` +
  `is_token` on the attack-trigger filter. Both are asserted by existing defs/markers but must
  be pinned by the per-card tests (the flying-filter and token/power-filter negative cases).
- **`exclude_self` on the ETB filter**: ogre_battledriver needs `exclude_self: true`
  (Ogre's own ETB must not fire); scourge/dragon-tempest need `exclude_self: false` (self and
  other both count). The per-card tests assert the boundary.
- **Wire double-bump**: PROTOCOL (DSL shape) and HASH (GameState closure) both move; keep them
  separate per SR-8 and append both history rows. Expect the two hash digests
  (`decl_fingerprint` + `stream_fingerprint`) to both move (new `EffectFilter` disc + new
  `Effect` field in the byte stream).
- **shared_animosity partial-double-blocker trap** (per feedback_verify_full_chain): PB-EF4
  closes only the *subject* gap; the *count* gap remains, so authoring it Complete would ship
  wrong game state (+0 buff every time). Keep it `inert`; file OOS-EF4-1. Do NOT substitute a
  gated `Effect::Choose`/fixed count.
