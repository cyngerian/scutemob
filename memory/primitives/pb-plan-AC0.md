# Primitive Batch Plan: PB-AC0 — ETBTriggerFilter subtype/nontoken filter forwarding on the creature-ETB trigger path

**Generated**: 2026-05-18
**Task**: scutemob-41
**Branch**: feat/pb-ac0-etbtriggerfilter-subtypenontoken-fields-creature-etb-
**Primitive**: Bring the `WheneverCreatureEntersBattlefield` (Alliance / creature-ETB)
trigger path to filter parity with the already-correct death-trigger path, so that
`has_subtype` / `has_subtypes` / `is_nontoken` (and other `TargetFilter` fields) on a
creature-ETB trigger are honored instead of silently dropped.
**CR Rules**: 603.2 (trigger event matching), 603.6a (ETB triggered abilities — *not*
look-back-in-time), 603.10 (the look-back exception list — ETB is NOT on it), 111.1
(token = a permanent not represented by a card → defines nontoken), 205.3 (subtypes),
613.1d / 613.4c (layer-resolved card types & subtypes — animated/granted creatures),
400.7 (object identity on zone change).
**Cards affected**: 6 total
  - 2 re-authored (live abilities, TODOs removed): `ganax_astral_hunter`, `lathliss_dragon_queen`
  - 3 latent over-trigger bugs fixed with **zero card-def edits** (already author live
    subtype-filtered creature-ETB triggers): `miirym_sentinel_wyrm`, `dragons_hoard`,
    `bloomvine_regent`
  - 1 forced add from TODO sweep (`is_nontoken` filter added + a second latent
    `EffectTarget` bug corrected): `the_great_henge`
**Dependencies**: none. `triggering_creature_filter` already exists on `TriggeredAbilityDef`
(added in PB-N), is already hashed, and `matches_filter` already checks `has_subtype` /
`has_subtypes` / `exclude_subtypes`. This batch only *populates and reads* an existing field.
**Deferred items from prior PBs**: none directly. This batch is the engine fix the
`scutemob-40` card review (`memory/card-authoring/review-scutemob-40.md`, F1/F3)
explicitly deferred ("authoring-only batch — cannot make the engine change").

---

## TODO Sweep (roster-recall gate)

Ran across `crates/engine/src/cards/defs/`:
- `Grep "TODO.*[Ss]ubtype | TODO.*nontoken | TODO.*over-trigger | TODO.*Dragon"`
- `Grep "ETBTriggerFilter | WheneverCreatureEntersBattlefield"`

Findings, classified against what PB-AC0 actually ships (honoring `TargetFilter` on the
creature-ETB path — subtype, nontoken/token, and any other `matches_filter`-checked field):

- **`the_great_henge.rs` — FORCED ADD.** File-header TODO line 8 + inline TODO line 40:
  *"'nontoken creature' filter — TargetFilter lacks non_token field. Using unfiltered
  creature trigger (includes tokens — slightly wrong)."* The TODO's *reason* is now stale —
  `TargetFilter.is_nontoken` **does** exist (`card_definition.rs:2621`); the real residual
  gap is that the creature-ETB path silently drops it. PB-AC0 closes exactly that gap, so
  Great Henge's `is_nontoken: true` filter becomes honorable. *Added via pre-existing TODO
  sweep — not in original PB brief.* (See "Card Definition Fixes" — Great Henge also carries
  a **second, unrelated latent bug**: it uses `EffectTarget::Source` for the +1/+1 counter,
  which puts the counter on Henge itself, not the entering creature. Corrected here too
  since the file is being touched and the oracle is unambiguous.)

- **`guardian_project.rs` — verify-only, NOT a forced add, TODO stays.** Header TODO names
  the same nontoken gap, but Guardian Project *also* needs an intervening-if
  ("if it doesn't have the same name as another creature you control or a creature card in
  your graveyard"). The name-uniqueness condition is a genuine separate DSL gap. Even with
  the nontoken filter honored, the card still produces wrong game state (draws on a
  duplicate-name creature). Per W5 it must stay as an over-broad approximation with its TODO
  intact. PB-AC0 may *optionally* tighten its filter to `is_nontoken: true` (strictly
  improves correctness — removes the token over-trigger) while leaving the name-uniqueness
  TODO; that is a judgement call for the runner. **Recommended: add `is_nontoken: true`**
  (it is a pure correctness improvement and the filter field is free) **but keep the
  name-uniqueness TODO.** Do NOT count Guardian Project as "unblocked."

- **`scourge_of_valkas.rs` — NOT a forced add.** TODO names "this creature OR another
  Dragon ETB" + `EffectAmount::CountCreaturesYouControl(filter)`. Both are genuine gaps
  unrelated to filter forwarding. PB-AC0 does not unblock it. Leave as-is.

- **`encroaching_dragonstorm.rs` / `roiling_dragonstorm.rs` — NOT forced adds.** Their
  Dragon-ETB triggers are blocked on `Effect::ReturnToHand { target: Source }`, a missing
  *effect* variant, not a filter gap. PB-AC0 does not unblock them. Leave their TODOs.
  (`encroaching_dragonstorm` is in the brief's "verify" list — verdict: correctly blocked on
  a *different* primitive; no change, no over-trigger today because the trigger is not
  authored at all. Documented in "Verification of brief's latent-bug cards" below.)

- `foundry_street_denizen.rs`, `tainted_observer.rs` — appear in the
  `WheneverCreatureEntersBattlefield` grep but are unrelated (Denizen uses only the
  color filter, which already works; Tainted Observer is a `MayPayOrElse` gap).

**TODO sweep result: 1 forced add (`the_great_henge.rs`).** Confirmed-yield count after
sweep: re-authored 2 + latent-fix-no-edit 3 + forced add 1 = **6 cards**.

---

## CR Rule Text (from MCP)

- **CR 603.2** — "Whenever a game event or game state matches a triggered ability's trigger
  event, that ability automatically triggers. The ability doesn't do anything at this point."
- **CR 603.10** — "Normally, objects that exist immediately after an event are checked to see
  if the event matched any trigger conditions, and continuous effects that exist at that time
  are used to determine what the trigger conditions are and what the objects involved in the
  event look like. However, some triggered abilities are exceptions to this rule [the game
  'looks back in time']. The list of exceptions is as follows:" — 603.10a leaves-the-
  battlefield / leaves-graveyard / put-into-hand-or-library; 603.10b phase out; 603.10c
  becomes unattached; 603.10d loses control; 603.10e countered; 603.10f loses the game;
  603.10g planeswalks away. **ETB triggers are NOT on this list.** ⇒ A creature-ENTERS-the-
  battlefield trigger evaluates the entering permanent's characteristics **as they exist
  immediately after it enters** (CR 603.10 default). This is why the matching loop correctly
  calls `calculate_characteristics(state, entering_id)` on the live entering object — the
  layer-resolved characteristics at entry time. No LKI / pre-event snapshot is needed here
  (unlike the death path, which is 603.10a and uses `pre_death_characteristics`).
- **CR 111.1** — "A token is a marker used to represent any permanent that isn't represented
  by a card." ⇒ "nontoken" = a permanent that *is* represented by a card. Engine encoding:
  `GameObject.is_token: bool`. This is a runtime field, NOT in `Characteristics`, so
  `matches_filter(&Characteristics, &TargetFilter)` cannot and does not check it — it must be
  checked explicitly at the call site (exactly as the death path and combat-damage path
  already do).
- **CR 205.3** — subtypes; creature types are a subtype set. `Characteristics.subtypes:
  OrdSet<SubType>`; `matches_filter` checks `has_subtype` (single), `has_subtypes` (OR-list),
  `exclude_subtypes` (AND-exclusion).
- **CR 603.6d** — "[This permanent] enters with…/as…/tapped" is a *static* ability, not a
  triggered ability — irrelevant here but confirms the entering permanent's enters-with
  state (counters, tapped) is settled before the trigger is checked.

**Lathliss ruling (Gatherer, 2024-11-08)**: "If Lathliss enters at the same time as one or
more other nontoken Dragons you control, its second ability will trigger once for each of
those other Dragons." ⇒ confirms `exclude_self: true` is correct for Lathliss (it does NOT
trigger off its own entry) and the trigger fires per-Dragon (the engine already dispatches
one `AnyPermanentEntersBattlefield` event per entering permanent — no change needed).

---

## The Verified Gap (full dispatch chain walk)

Chain walked end-to-end; line numbers verified against the worktree, not trusted from the brief.

1. **Card DSL** — `TriggerCondition::WheneverCreatureEntersBattlefield { filter:
   Option<TargetFilter>, exclude_self: bool }`. `TargetFilter` (`card_definition.rs:2548`)
   carries `has_subtype`, `has_subtypes`, `is_nontoken`, `is_token`, `exclude_subtypes`,
   etc.

2. **Harness conversion** — `enrich_spec_from_def` in
   `crates/engine/src/testing/replay_harness.rs`, the `WheneverCreatureEntersBattlefield`
   loop at **lines 2360-2408**. It builds an `ETBTriggerFilter` (lines 2371-2392) that
   copies ONLY `creature_only` (hardcoded `true`), `controller_you`, `exclude_self`,
   `color_filter` (single-color only), and `card_type_filter` (hardcoded `None`). It then
   sets `triggering_creature_filter: None` (line 2405). **The carddef `filter`'s
   `has_subtype` / `has_subtypes` / `is_nontoken` / `is_token` / `exclude_subtypes` are
   never read.** ← THE DROP SITE.

3. **`ETBTriggerFilter` struct** — `crates/engine/src/state/game_object.rs:560-577`. Five
   fields only: `creature_only`, `controller_you`, `exclude_self`, `color_filter`,
   `card_type_filter`. No subtype field, no token field.

4. **Matching loop** — `collect_triggers_for_event` in `crates/engine/src/rules/abilities.rs`,
   the `etb_filter` block at **lines 6142-6192**. Checks exactly those five fields against
   `calculate_characteristics(state, entering_id)`. Has no subtype/token check.

5. **Death-path comparison (the correct reference implementation)** — the `AnyCreatureDies`
   block in `abilities.rs` at **lines 4287-4376**. It checks `death_filter` (controller/
   exclude_self/nontoken — `DeathTriggerFilter`) AND, crucially, checks
   `trigger_def.triggering_creature_filter` (a full `TargetFilter`) at lines 4346-4376:
   an explicit `creature_filter.is_token && !dying_is_token` guard, then
   `matches_filter(&dying_chars, creature_filter)` for subtype/type/etc. `triggering_
   creature_filter` is forwarded by the death-trigger harness conversion. The combat-damage
   path (`abilities.rs:6093-6129`) does the same with both `combat_damage_filter` and
   `triggering_creature_filter`.

**Conclusion**: the creature-ETB path is the *only* one of the three creature-event paths
(ETB / death / combat-damage) that fails to forward the carddef `TargetFilter`. The fix is
to make it forward, mirroring death.

---

## Shape Decision: Approach (b) — forward `triggering_creature_filter`

Two approaches were considered (per the brief):

**(a)** Add `has_subtype` / `has_subtypes` / `exclude_subtypes` / `is_nontoken` / `is_token`
fields to `ETBTriggerFilter` and honor them in the `abilities.rs` ETB matching loop.

**(b)** Have the creature-ETB harness conversion forward the carddef `TargetFilter` whole as
`triggering_creature_filter` (an already-existing field on `TriggeredAbilityDef`), and add a
`triggering_creature_filter` check to the `abilities.rs` ETB matching block — mirroring the
already-correct death-trigger path.

### Decision: **(b)**. Rationale:

1. **Zero new struct fields ⇒ zero hash-schema risk.** `triggering_creature_filter:
   Option<TargetFilter>` already exists on `TriggeredAbilityDef` (`game_object.rs:614-621`,
   added by PB-N) and is **already hashed** (`hash.rs:2523-2524`,
   `self.triggering_creature_filter.hash_into(hasher)`). Approach (b) only changes which
   `TriggeredAbilityDef` instances populate that field — the wire format of the struct does
   not change at all. **No `HASH_SCHEMA_VERSION` bump required** (see "Hash Impact" below).
   Approach (a) adds ≥4 fields to `ETBTriggerFilter`, which IS hashed
   (`hash.rs:2494-2503`) — that mandates a hash bump + sentinel sweep.

2. **Mirrors the proven death path exactly.** The death block already does
   `is_token` guard + `matches_filter`. Approach (b) is a near-verbatim transplant of
   lines 4346-4376 into the ETB block — lowest-novelty, lowest-risk change. The reviewer
   recommended this shape explicitly (review-scutemob-40.md F1: "forward
   `triggering_creature_filter` on the creature-ETB path, mirroring the death path").

3. **`matches_filter` already does the subtype work.** `has_subtype`, `has_subtypes`,
   `exclude_subtypes`, plus colors/types/power/keywords — all already implemented in
   `matches_filter` (`effects/mod.rs:7244-7371`). Approach (a) would duplicate a subset of
   that logic inline in the ETB loop. Approach (b) reuses it whole and, as a bonus, makes
   *every* `TargetFilter` field work on the ETB path for free (e.g. a future "whenever a
   legendary creature you control enters" needs no further engine work).

4. **`is_token` / `is_nontoken` parity.** These are `GameObject` runtime fields, not in
   `Characteristics`; `matches_filter` cannot see them. Both the death path and the
   combat-damage path handle this with an *explicit* call-site check. Approach (b) adds the
   identical explicit check to the ETB loop — `is_nontoken` for Lathliss/Miirym/Great Henge,
   `is_token` for completeness. (`ETBTriggerFilter` itself still gains nothing.)

**`ETBTriggerFilter` is retained unchanged** — it still carries `creature_only` /
`controller_you` / `exclude_self` / `color_filter` / `card_type_filter`, all of which keep
working. The two filters coexist on the same `TriggeredAbilityDef` for the
creature-ETB conversion: `etb_filter` keeps doing `exclude_self` + `creature_only` +
`controller_you` + `color_filter`; the new `triggering_creature_filter` adds subtype /
nontoken / and anything else. (color is harmlessly checkable by both — see Risks.)

---

## Engine Changes

### Change 1 — Harness: forward the carddef `TargetFilter` on the creature-ETB path

**File**: `crates/engine/src/testing/replay_harness.rs`
**Site**: the `WheneverCreatureEntersBattlefield` conversion loop, **lines 2360-2408**.
**Action**: in the `TriggeredAbilityDef { … }` struct literal built at lines 2393-2407,
change the `triggering_creature_filter: None` (line 2405) to forward the carddef filter:

```rust
// PB-AC0: forward the full carddef TargetFilter as triggering_creature_filter so
// has_subtype / has_subtypes / is_nontoken / exclude_subtypes are honored — the
// ETBTriggerFilter above only carries creature_only/controller_you/exclude_self/
// color_filter/card_type_filter. Mirrors the death-trigger conversion. CR 603.2.
triggering_creature_filter: filter.clone(),
```

`filter` here is the `&Option<TargetFilter>` bound by the `TriggerCondition::
WheneverCreatureEntersBattlefield { filter, exclude_self }` destructure at lines 2362-2366.
`triggering_creature_filter` is typed `Option<TargetFilter>`, so `filter.clone()` is the
exact type — no wrapping needed.

**Leave `etb_filter` exactly as-is.** Both filters now ride on the same
`TriggeredAbilityDef`. `etb_filter` handles `exclude_self` (the "another" qualifier — the
self-exclusion logic) + `creature_only`; `triggering_creature_filter` handles subtype /
nontoken. They are AND-combined by the matching loop (both must pass). The brief's
death-path note confirms this is the established dual-filter pattern (death uses
`death_filter` for controller/exclude_self/nontoken AND `triggering_creature_filter` for
subtype).

> Note for the runner: there is no separate "death-trigger harness conversion" function to
> copy — the death path's `triggering_creature_filter` is populated in the same
> `enrich_spec_from_def`. Grep `triggering_creature_filter:` in `replay_harness.rs` to see
> the death/attack sites that already forward it (they forward via the death-filter
> conversion loop). The change here is one line.

### Change 2 — Matching loop: honor `triggering_creature_filter` on the ETB path

**File**: `crates/engine/src/rules/abilities.rs`
**Site**: `collect_triggers_for_event`, the `etb_filter` block at **lines 6142-6192**.
**Action**: add a `triggering_creature_filter` check. Place it **after** the `etb_filter`
block (after the closing `}` at line 6192) and **before** the intervening-if check at
line 6195, so it runs only for ETB-class events. Mirror the death-path block at
`abilities.rs:4346-4376`:

```rust
// PB-AC0 (CR 603.2 / CR 205.3 / CR 111.1): honor triggering_creature_filter on the
// creature-ETB path — subtype / nontoken / exclude_subtypes / and any other
// matches_filter-checked constraint on the entering creature. Mirrors the
// AnyCreatureDies block (abilities.rs ~4346) and the combat-damage block (~6113).
// The ETBTriggerFilter above handles exclude_self/creature_only/controller_you/
// color_filter/card_type_filter; this handles the rest.
//
// CR 603.10: ETB is NOT a look-back-in-time trigger — the entering permanent's
// characteristics are evaluated as they exist immediately after entry, so we use
// calculate_characteristics on the live entering object (no LKI snapshot).
if let Some(ref creature_filter) = trigger_def.triggering_creature_filter {
    if let Some(entering_id) = entering_object {
        if let Some(entering_obj) = state.objects.get(&entering_id) {
            // is_token / is_nontoken: GameObject runtime fields, not in
            // Characteristics — checked explicitly (matches_filter cannot see them).
            if creature_filter.is_token && !entering_obj.is_token {
                continue;
            }
            if creature_filter.is_nontoken && entering_obj.is_token {
                continue;
            }
            // CR 613.1d (Layer 4): layer-resolved subtypes/types so animated or
            // type-/subtype-granted permanents match correctly.
            let entering_chars =
                crate::rules::layers::calculate_characteristics(state, entering_id)
                    .unwrap_or_else(|| entering_obj.characteristics.clone());
            if !crate::effects::matches_filter(&entering_chars, creature_filter) {
                continue;
            }
        } else {
            // Entering object not found — skip conservatively.
            continue;
        }
    } else {
        // triggering_creature_filter set but no entering object — skip.
        continue;
    }
}
```

**Guard the scope.** The `etb_filter` block at 6142 already runs for ETB-class events
(`etb_filter` is only ever `Some` on ETB-converted defs). `triggering_creature_filter` is
*also* populated on attack/death/combat-damage defs (PB-N) — but the attack/damage block
(lines 6116-6129) and the death block (4346-4376) already consume it on *their* paths. To
avoid double-consumption / wrong-context evaluation, the new block must only fire for the
ETB event class. Two safe options for the runner (pick one, prefer the first):

  - **(preferred) Gate on `trigger_def.etb_filter.is_some()`**: only run the new block when
    the def also has an `etb_filter`. Every creature-ETB-converted def sets `etb_filter:
    Some(..)` (the harness always builds one at lines 2371-2392, even when the carddef
    `filter` is `None`). Attack/death/damage defs never set `etb_filter`. This cleanly
    scopes the new check to ETB defs only. Wrap the new block in
    `if trigger_def.etb_filter.is_some() { … }` or fold the
    `triggering_creature_filter` check *inside* the existing `if let Some(ref etb_filter)`
    block (after the `card_type_filter` check at line 6181, before its closing braces) —
    folding inside is cleanest and is the recommended placement.
  - (alternative) Gate on `event_type` being an ETB event
    (`AnyPermanentEntersBattlefield`). The attack/damage block at 6076-6138 is already
    gated by an `if event_type == …` ladder; the death block is in a different dispatch
    function entirely (`abilities.rs:4287`, not `collect_triggers_for_event`). So an
    `event_type`-based gate also works, but the `etb_filter.is_some()` gate is simpler.

  **RUNNER ACTION**: place the new `triggering_creature_filter` check **inside** the
  existing `if let Some(ref etb_filter) = trigger_def.etb_filter` block (lines 6142-6192),
  immediately after the `card_type_filter` check (after line 6181) and before the block's
  closing braces. This guarantees it only runs for ETB-converted defs, reuses the already-
  computed `entering_id` / `entering_obj` / `entering_chars` from that block (no recompute),
  and keeps the death/attack paths untouched. The code sketch above shows it as a standalone
  block for clarity; **the runner should inline it into the `etb_filter` block reusing the
  existing `entering_chars`** rather than recomputing `calculate_characteristics`.

**Verify the death/attack paths are untouched**: with the `etb_filter.is_some()` scoping,
`AnyCreatureDies` (handled in a different function, `abilities.rs:4287`) and
`AnyCreatureYouControlAttacks` / `…DealsCombatDamageToPlayer` (handled at 6076-6138, those
defs have `etb_filter: None`) never enter the new block. Their existing
`triggering_creature_filter` handling at 4350 and 6116 is unchanged.

### Change 3 — `ETBTriggerFilter`: no change

Approach (b) does not modify `ETBTriggerFilter`. The struct, its `HashInto` impl
(`hash.rs:2494-2503`), and its `Default` derive are all untouched.

---

## Hash Impact Assessment

**No `HASH_SCHEMA_VERSION` bump required.**

- `HASH_SCHEMA_VERSION` is currently **27** (`crates/engine/src/state/hash.rs:202` —
  `pub const HASH_SCHEMA_VERSION: u8 = 27;`; last bumped by the LOW-Sweep campaign, NOT 26 —
  the WIP file's "PB-LS6 bumped 25→26" is one bump behind the current tree).
- The only `TriggeredAbilityDef` field this batch newly *populates* is
  `triggering_creature_filter`. That field **already exists** on the struct
  (`game_object.rs:614-621`) and is **already hashed** unconditionally
  (`hash.rs:2512-2525`, line 2524: `self.triggering_creature_filter.hash_into(hasher);`).
- No struct gains or loses a field. No enum gains a variant. `ETBTriggerFilter` is
  unchanged. `TargetFilter` is unchanged. `Effect` is unchanged.
- The *value* hashed for an affected card's `TriggeredAbilityDef` changes (Ganax/Lathliss/
  Miirym/etc. now hash a `Some(TargetFilter)` where they previously hashed `None`) — but
  that is a normal consequence of any card-def correction, not a wire-format / schema
  change. State hashes of *games using those cards* differ from old recordings; that is
  expected and correct (the cards now behave differently). It does NOT require a
  `HASH_SCHEMA_VERSION` bump — the schema (which fields are hashed, in what order) is
  identical.

**Standing rule check** (`hash.rs` module comment, ~line 36-43, and the inline note at
~6539-6546: *"bump on any wire-format-affecting change to TriggeredAbilityDef"*): this batch
makes **no wire-format change** to `TriggeredAbilityDef` — same fields, same order, same
`HashInto`. ⇒ rule does not trigger. **Do not bump. Do not touch the
`assert_eq!(HASH_SCHEMA_VERSION, 27)` parity test.**

> STOP-AND-FLAG for the runner: if, during implementation, you find you must add a field to
> `ETBTriggerFilter` or `TargetFilter` after all (you should not — approach (b) is
> field-free), STOP and escalate. That would change the decision and mandate a 27→28 bump
> plus the parity-test/module-comment update. The plan is explicitly approach (b) precisely
> to avoid this.

---

## Card Definition Fixes

### 1. `crates/engine/src/cards/defs/ganax_astral_hunter.rs` — re-author (TODO removed)

**Oracle (MCP)**: "Flying / Whenever Ganax or another Dragon you control enters, create a
Treasure token. / Choose a Background"

**Current state**: ENGINE-BLOCKED — the Dragon-ETB Treasure trigger was reverted to a
TODO comment (lines 18-26) in `scutemob-40`; only `Flying` + `ChooseABackground` keywords
are live.

**Fix**: delete the TODO comment block (lines 18-26), insert a live trigger between the
`Flying` and `ChooseABackground` keyword entries:

```rust
// CR 603.2: "Whenever Ganax or another Dragon you control enters, create a Treasure token."
// "Ganax OR another Dragon" = any Dragon you control, including itself → exclude_self: false.
AbilityDefinition::Triggered {
    trigger_condition: TriggerCondition::WheneverCreatureEntersBattlefield {
        filter: Some(TargetFilter {
            has_subtype: Some(SubType("Dragon".to_string())),
            controller: TargetController::You,
            ..Default::default()
        }),
        exclude_self: false,
    },
    effect: Effect::CreateToken { spec: treasure_token_spec(1) },
    intervening_if: None,
    targets: vec![],
    modes: None,
    trigger_zone: None,
},
```

`exclude_self: false` — oracle says "Ganax **or** another Dragon", so Ganax's own entry
fires it. `treasure_token_spec(1)` is the established Treasure helper (used by
`prosperous_innkeeper`, `smothering_tithe`, etc.). Update the file-header comment if it
still references the gap.

### 2. `crates/engine/src/cards/defs/lathliss_dragon_queen.rs` — re-author (TODO removed)

**Oracle (MCP)**: "Flying / Whenever another nontoken Dragon you control enters, create a
5/5 red Dragon creature token with flying. / {1}{R}: Dragons you control get +1/+0 until
end of turn."

**Current state**: PARTIAL — ETB token trigger is an ENGINE-BLOCKED TODO (lines 19-31); the
`{1}{R}` pump activated ability (lines 32-50) is live and verified correct by the review
(leave it untouched).

**Fix**: delete the TODO comment block (lines 19-31), insert a live trigger before the
existing `Activated` pump ability:

```rust
// CR 603.2: "Whenever another nontoken Dragon you control enters, create a 5/5 red
// Dragon creature token with flying." exclude_self: true ("another"); is_nontoken: true
// ("nontoken Dragon"). Per Gatherer ruling 2024-11-08, fires once per other nontoken
// Dragon entering simultaneously — the engine dispatches one ETB event per entrant.
AbilityDefinition::Triggered {
    trigger_condition: TriggerCondition::WheneverCreatureEntersBattlefield {
        filter: Some(TargetFilter {
            has_subtype: Some(SubType("Dragon".to_string())),
            controller: TargetController::You,
            is_nontoken: true,
            ..Default::default()
        }),
        exclude_self: true,
    },
    effect: Effect::CreateToken { spec: <5/5 red Dragon with flying token spec> },
    intervening_if: None,
    targets: vec![],
    modes: None,
    trigger_zone: None,
},
```

**Token spec**: a 5/5 red Dragon creature token with flying. There is no `dragon_token_spec`
helper. The runner builds a `TokenSpec` inline. Reference patterns: `sarkhan_unbroken.rs`
(4/4 red Dragon token with flying), `sarkhan_fireblood.rs` (5/5 red Dragon tokens with
flying — exact P/T match), `goro_goro_disciple_of_ryusei.rs` (5/5 red Dragon Spirit with
flying). **Copy the `TokenSpec` literal from `sarkhan_fireblood.rs`'s −7 ability** (it is
the exact "5/5 red Dragon creature token with flying" the runner needs) and use count 1.
Verify the `TokenSpec` fields against `card_definition.rs` (`treasure_token_spec` /
`TokenSpec` definition near line 3378) — typically `name`, `colors`, `types`/`subtypes`,
`power`/`toughness`, `keywords` (Flying), `count`.

`exclude_self: true` — "**another** nontoken Dragon", so Lathliss's own ETB does not fire
it (review F3 + Gatherer ruling confirm). `is_nontoken: true` — the new engine check skips
token Dragons. `has_subtype: Dragon` — the new `matches_filter` check restricts to Dragons.

### 3. `crates/engine/src/cards/defs/the_great_henge.rs` — forced add (nontoken filter + EffectTarget bug)

**Oracle (MCP)**: "… / Whenever a nontoken creature you control enters, put a +1/+1 counter
on it and draw a card."

**Current state**: the creature-ETB trigger (lines 41-65) is live but **doubly wrong**:
  (i) filter is unfiltered (`controller: You` only) → over-triggers on token creatures;
  (ii) the +1/+1 counter uses `EffectTarget::Source` (line 51) → puts the counter on The
       Great Henge itself, not on the entering creature. Oracle says "put a +1/+1 counter
       on **it**" where "it" = the entering nontoken creature. **This is a second latent
       bug, independent of PB-AC0's primitive**, but the file is being edited and the
       oracle is unambiguous, so correct it here.

**Fix**:
  - Add `is_nontoken: true` to the `TargetFilter` (the new engine check now honors it).
  - Change the `AddCounter` target from `EffectTarget::Source` to
    `EffectTarget::TriggeringCreature` (resolves via `ctx.triggering_creature_id` ←
    `PendingTrigger::entering_object_id`; confirmed at `effects/mod.rs:6116`).
  - Delete the stale inline TODO at line 40 and the file-header TODO at line 8 (the
    nontoken half is now resolved). **Keep** the line 6-7 header TODO about
    `SelfCostReduction::GreatestPowerAmongCreatures` — that cost-reduction gap is unrelated
    and still genuine.

Resulting trigger:

```rust
// CR 603.2: "Whenever a nontoken creature you control enters, put a +1/+1 counter on
// it and draw a card." is_nontoken honored via PB-AC0; counter goes on the entering
// creature (TriggeringCreature), not on Henge.
AbilityDefinition::Triggered {
    trigger_condition: TriggerCondition::WheneverCreatureEntersBattlefield {
        filter: Some(TargetFilter {
            controller: TargetController::You,
            is_nontoken: true,
            ..Default::default()
        }),
        exclude_self: false,
    },
    effect: Effect::Sequence(vec![
        Effect::AddCounter {
            target: EffectTarget::TriggeringCreature,
            counter: CounterType::PlusOnePlusOne,
            count: 1,
        },
        Effect::DrawCards {
            player: PlayerTarget::Controller,
            count: EffectAmount::Fixed(1),
        },
    ]),
    intervening_if: None,
    targets: vec![],
    modes: None,
    trigger_zone: None,
},
```

> The `EffectTarget::Source → TriggeringCreature` correction is flagged for the
> primitive-impl-reviewer as a deliberate, in-scope second fix (oracle-unambiguous, same
> file). If the reviewer judges it out-of-scope, it can be split — but leaving a wrong
> `EffectTarget` in a file being actively edited would itself be a review finding.

### 4-6. Latent over-trigger fixes — verified, **no card-def edits**

These three already author live `WheneverCreatureEntersBattlefield` triggers with a
subtype filter; Change 1 + Change 2 fix them automatically. The runner must **verify** each
compiles and behaves, but must NOT edit them (no edit is needed — the carddef is already
correct; only the engine was dropping the filter).

- **`miirym_sentinel_wyrm.rs`** (lines 26-48). Oracle (MCP): "… / Whenever another nontoken
  Dragon you control enters, create a token that's a copy of it, except the token isn't
  legendary." Current def: `WheneverCreatureEntersBattlefield` with
  `filter: { has_subtype: Dragon, controller: You }`, `exclude_self: true`,
  `effect: CreateTokenCopy { source: TriggeringCreature, except_not_legendary: true, … }`.
  **Today it over-triggers**: subtype dropped → copies *any* creature; nontoken not even
  expressed. **After PB-AC0**: `has_subtype: Dragon` is honored. **Card-def correction
  REQUIRED but minimal**: the def is missing `is_nontoken: true` (oracle says "nontoken
  Dragon") — the TODO at line 25 says *"TargetFilter lacks 'nontoken' restriction; currently
  fires on token Dragons too."* That TODO's *reason* is stale (`is_nontoken` exists). The
  runner SHOULD add `is_nontoken: true` to Miirym's `TargetFilter` and delete the line-25
  TODO — this is the same one-field add as Lathliss and is now honored. (The line-42 TODO
  about `SourceIsNotSelf` is obsolete too — `exclude_self: true` already handles "another";
  delete it.) ⇒ **Miirym IS a small card-def edit after all** — reclassify it from
  "no-edit" to "one-field edit": add `is_nontoken: true`, remove two stale TODOs.

- **`dragons_hoard.rs`** (lines 16-35). Oracle (MCP): "Whenever a Dragon you control enters,
  put a gold counter on this artifact. …" Current def: `WheneverCreatureEntersBattlefield`
  with `filter: { has_subtype: Dragon, controller: You }`, `exclude_self: false`.
  **Today over-triggers** (gold counter on every creature). **After PB-AC0**: correct — fires
  only for Dragons. Oracle has no "nontoken" word ⇒ no `is_nontoken`. **No card-def edit
  needed** — verify only.

- **`bloomvine_regent.rs`** (lines 21-39). Oracle (MCP): "Flying / Whenever this creature or
  another Dragon you control enters, you gain 3 life." Current def:
  `WheneverCreatureEntersBattlefield` with `filter: { has_subtype: Dragon, controller: You }`,
  `exclude_self: false`. **Today over-triggers** (gain 3 life per any creature). **After
  PB-AC0**: correct — "this or another Dragon" = any Dragon you control incl. self ⇒
  `exclude_self: false` ✓, `has_subtype: Dragon` now honored. **No card-def edit needed** —
  verify only.

### Verification of brief's latent-bug cards — summary

| Card | Brief asked | Verdict | Action |
|------|-------------|---------|--------|
| `miirym_sentinel_wyrm` | verify/correct | Over-triggers today; subtype now honored; oracle says "nontoken" | **Edit**: add `is_nontoken: true`; delete 2 stale TODOs |
| `dragons_hoard` | verify/correct | Over-triggers today; fixed by engine change | **No edit**; verify |
| `bloomvine_regent` | verify/correct | Over-triggers today; fixed by engine change | **No edit**; verify |
| `encroaching_dragonstorm` | verify/correct | Dragon-ETB trigger NOT authored (blocked on `Effect::ReturnToHand`, a *different* gap); no over-trigger today | **No edit**; leave TODO; note PB-AC0 does not unblock it |

So the brief's four "verify" cards resolve to: 2 no-edit (Dragon's Hoard, Bloomvine Regent),
1 one-field edit (Miirym), 1 unaffected/different-gap (Encroaching Dragonstorm).

---

## Exhaustive-Match / Wiring Checklist

| File | Site | Action |
|------|------|--------|
| `crates/engine/src/testing/replay_harness.rs` | `WheneverCreatureEntersBattlefield` conversion, ~L2405 | Change `triggering_creature_filter: None` → `filter.clone()` |
| `crates/engine/src/rules/abilities.rs` | `collect_triggers_for_event`, inside `etb_filter` block ~L6181 | Add `triggering_creature_filter` check (subtype + is_token/is_nontoken + matches_filter) |
| `crates/engine/src/cards/defs/ganax_astral_hunter.rs` | abilities vec | Replace TODO with live `WheneverCreatureEntersBattlefield` trigger |
| `crates/engine/src/cards/defs/lathliss_dragon_queen.rs` | abilities vec | Replace TODO with live trigger (5/5 red Dragon token, `is_nontoken`, `exclude_self`) |
| `crates/engine/src/cards/defs/the_great_henge.rs` | trigger ~L41 | Add `is_nontoken: true`; `Source`→`TriggeringCreature`; delete 2 stale TODOs |
| `crates/engine/src/cards/defs/miirym_sentinel_wyrm.rs` | trigger ~L28 | Add `is_nontoken: true`; delete 2 stale TODOs |

**No enum variant added, no struct field added.** ⇒ **no exhaustive-match breakage.**
`Effect`, `TriggerEvent`, `StackObjectKind`, `KeywordAbility`, `ETBTriggerFilter`,
`TargetFilter`, `TriggeredAbilityDef` are all structurally unchanged. Confirmed: no edits
needed in `state/hash.rs`, `tools/tui/`, `tools/replay-viewer/`. Still run
`cargo build --workspace` after the implement phase per the standing "runners miss
tools-crate exhaustive matches ~50% of the time" gotcha — but this batch adds no variant,
so the risk is near-zero.

---

## Unit Tests

**File**: `crates/engine/tests/etb_trigger_subtype_filter.rs` (new file).
**Pattern**: follow existing creature-ETB / Alliance trigger tests — grep `tests/` for
`WheneverCreatureEntersBattlefield` or `Impact Tremors` / `Prosperous Innkeeper` test
files and mirror the multi-player builder setup (use ≥2 players so turns advance and ETB
events dispatch normally). Mirror the death-subtype tests
(grep `triggering_creature_filter` in `tests/` — there will be PB-N death-filter tests to
copy structure from).

Tests (each cites the CR rule it validates):

- `test_etb_subtype_filter_fires_on_match` — controller has Ganax-style trigger
  (`has_subtype: Dragon`); a Dragon creature enters under their control; assert the trigger
  fires (Treasure / gold counter / life-gain effect observed). **CR 603.2 / 205.3.**
- `test_etb_subtype_filter_no_fire_on_mismatch` — same trigger; a non-Dragon creature (e.g.
  a Goblin) enters under their control; assert the trigger does NOT fire (was the
  over-trigger bug). **CR 603.2.**
- `test_etb_nontoken_filter_fires_on_nontoken` — Lathliss-style trigger
  (`has_subtype: Dragon, is_nontoken: true, exclude_self: true`); a *nontoken* Dragon
  enters; assert the 5/5 Dragon token is created. **CR 111.1.**
- `test_etb_nontoken_filter_no_fire_on_token` — same Lathliss-style trigger; a *token*
  Dragon enters under their control; assert the trigger does NOT fire (no second token —
  prevents an infinite token loop). **CR 111.1 / 603.2.**
- `test_etb_subtype_and_nontoken_combined` — token *non-Dragon*, token *Dragon*, nontoken
  *non-Dragon*, nontoken *Dragon* each enter; assert the Lathliss-style trigger fires for
  ONLY the nontoken Dragon. Covers the AND-combination of `has_subtype` + `is_nontoken`.
  **CR 603.2 / 205.3 / 111.1.**
- `test_etb_exclude_self_with_subtype` — Lathliss enters the battlefield itself (a Dragon);
  assert its own `exclude_self: true` trigger does NOT fire off its own entry. Then another
  nontoken Dragon enters; assert it fires once. **CR 603.2** (+ Gatherer ruling 2024-11-08).
- `test_etb_subtype_filter_layer_resolved` — a non-Dragon creature you control that is
  granted the Dragon subtype by a continuous effect enters; assert a `has_subtype: Dragon`
  ETB trigger fires for it (validates the `calculate_characteristics` layer-resolution path,
  CR 613.1d). If setting up a subtype-granting effect is too heavy, this may be downgraded
  to a colors-based variant (a color granted by a continuous effect) — but prefer the
  subtype version since subtype is the primitive under test.
- `test_etb_ganax_treasure_integration` — full card-def integration: Ganax on the
  battlefield, cast/resolve a Dragon creature; assert exactly one Treasure token is created.
  Then resolve a non-Dragon creature; assert no Treasure. **CR 603.2** (integration of the
  re-authored card).
- `test_etb_lathliss_token_integration` — Lathliss on the battlefield; a nontoken Dragon
  enters → assert one 5/5 red Dragon token with flying; a token creature enters → assert no
  new token. **CR 111.1** (integration of the re-authored card).
- `test_etb_great_henge_counter_on_entering_creature` — The Great Henge on the battlefield;
  a nontoken creature you control enters → assert the +1/+1 counter is on the *entering
  creature* (not on Henge) and a card was drawn; a token creature enters → assert no counter,
  no draw. **CR 603.2** (validates both the `is_nontoken` filter and the
  `EffectTarget::TriggeringCreature` correction).
- `test_etb_death_path_unaffected` — regression guard: a `WheneverCreatureDies` trigger with
  a `triggering_creature_filter` (subtype) still fires correctly for a matching death and
  not for a mismatch. Confirms Change 2's `etb_filter`-scoping did not disturb the death
  path. **CR 603.10a.** (If an equivalent death-subtype test already exists in the suite,
  this can be a pointer/comment instead of a duplicate.)

**Negative-path coverage is mandatory** — the bug being fixed is an *over*-trigger, so the
no-fire-on-mismatch tests (`…no_fire_on_mismatch`, `…no_fire_on_token`) are the load-bearing
assertions.

---

## Verification Checklist

- [ ] `replay_harness.rs` creature-ETB conversion forwards `filter.clone()` as
      `triggering_creature_filter`
- [ ] `abilities.rs` `collect_triggers_for_event` ETB block honors
      `triggering_creature_filter` (subtype via `matches_filter` + explicit
      `is_token`/`is_nontoken` guards), scoped to ETB defs (inside the `etb_filter` block)
- [ ] Death path (`abilities.rs:4287`) and attack/combat-damage path (`abilities.rs:6076`)
      verified untouched — no double-consumption of `triggering_creature_filter`
- [ ] `HASH_SCHEMA_VERSION` **NOT** bumped (still 27); parity test untouched — confirmed no
      struct/enum wire-format change
- [ ] `ganax_astral_hunter.rs` — live Dragon-ETB Treasure trigger; ENGINE-BLOCKED TODO removed
- [ ] `lathliss_dragon_queen.rs` — live nontoken-Dragon-ETB 5/5-token trigger; TODO removed;
      pump ability untouched
- [ ] `the_great_henge.rs` — `is_nontoken: true` added; `EffectTarget::Source` →
      `TriggeringCreature`; stale nontoken TODOs removed; cost-reduction TODO kept
- [ ] `miirym_sentinel_wyrm.rs` — `is_nontoken: true` added; 2 stale TODOs removed
- [ ] `dragons_hoard.rs`, `bloomvine_regent.rs` — verified correct, no edit
- [ ] New tests in `tests/etb_trigger_subtype_filter.rs` — fire-on-match + no-fire-on-mismatch
      for both subtype and nontoken; CR citations present
- [ ] `cargo check -p mtg-engine` clean
- [ ] `cargo build --workspace` clean (tools crates — near-zero risk, no variant added)
- [ ] `cargo test --all` passes (new tests + no regression in the 2860-test baseline)
- [ ] `cargo clippy --all-targets -- -D warnings` clean
- [ ] `cargo fmt --check` clean
- [ ] No remaining `TODO` referencing the ETBTriggerFilter subtype/nontoken drop in any of
      the 5 edited card-def files
- [ ] primitive-impl-reviewer pass run; findings addressed

---

## Risks & Edge Cases

- **Double-filter coexistence (`etb_filter` + `triggering_creature_filter`).** Both ride on
  the same creature-ETB `TriggeredAbilityDef`. `color` *could* be checked by both
  (`etb_filter.color_filter` AND `matches_filter` via `TargetFilter.colors`). This is
  harmless — they AND-combine and agree (both derive from the same carddef `filter`). No
  card is broken by the redundancy. Not worth de-duplicating; note it for the reviewer.

- **Scoping the new check to ETB defs.** `triggering_creature_filter` is *also* populated on
  attack/death defs by PB-N. The new block MUST be scoped (inside the `etb_filter` block, or
  gated on `etb_filter.is_some()`) so it does not double-consume the field on the
  attack/damage path (6076-6138) or run in the wrong context. The death path is in a
  different function (`abilities.rs:4287`) so it is naturally unaffected. `test_etb_death_
  path_unaffected` is the regression guard. This is the single highest-risk part of the
  change — get the placement right (inside the `etb_filter` block, after `card_type_filter`).

- **ETB is not look-back-in-time (CR 603.10).** The matching loop uses
  `calculate_characteristics` on the *live* entering object — correct, because the entering
  permanent's characteristics are evaluated as they exist immediately after entry. Do NOT
  add LKI/pre-event snapshot logic (that is the death path's concern, CR 603.10a). A
  creature that enters and is *then* (later) animated/changed does not retroactively un-fire
  or fire — entry is a single instant.

- **`is_token` field semantics.** `GameObject.is_token` is set at token creation; nontoken
  cards have `is_token: false`. The explicit guards (`is_token && !obj.is_token` reject;
  `is_nontoken && obj.is_token` reject) mirror the death path verbatim. A copy-token of a
  nontoken card (Miirym's own output!) has `is_token: true` — so Lathliss/Miirym do NOT
  chain off each other's tokens. This is correct per oracle ("nontoken") and is exactly the
  infinite-loop guard `test_etb_nontoken_filter_no_fire_on_token` checks.

- **Lathliss token spec.** No `dragon_token_spec` helper exists. The runner hand-builds the
  `TokenSpec` (5/5 red Dragon, Flying). Copy from `sarkhan_fireblood.rs` (−7: "five 5/5 red
  Dragon creature tokens with flying" — exact P/T and keyword match) and set count 1. Verify
  `TokenSpec` field names against `card_definition.rs:~3378`.

- **`EffectTarget::Source` → `TriggeringCreature` for Great Henge.** This is a *second*
  latent bug, technically outside the PB-AC0 primitive. It is corrected here because (a) the
  file is being edited for the `is_nontoken` filter anyway, (b) the oracle is unambiguous
  ("a +1/+1 counter on **it**"), and (c) leaving a known-wrong `EffectTarget` in an
  actively-edited file would itself be a reviewer finding. Flagged explicitly for the
  primitive-impl-reviewer as in-scope. If the reviewer disagrees, it is trivially splittable.

- **Yield calibration.** Per `feedback_pb_yield_calibration.md`, PB rosters historically
  overcount. This roster is conservatively scoped: 2 re-authored (brief), 1 one-field edit
  (Miirym — brief "verify" that turned out to need a field), 1 forced add (Great Henge —
  TODO sweep), 2 verified-no-edit (Dragon's Hoard, Bloomvine Regent — brief "verify"). All 6
  are confirmed by oracle-text + def-file inspection, not estimated. Encroaching Dragonstorm
  is explicitly *excluded* from the yield (different gap). No micro-PB spawn anticipated.

- **STOP-AND-FLAG concerns** (escalate to oversight if hit):
  1. If implementation reveals a field MUST be added to `ETBTriggerFilter` / `TargetFilter`
     — it should not, approach (b) is field-free — STOP: that flips the hash-bump decision.
  2. If `matches_filter` turns out to evaluate something context-sensitive that misbehaves
     for an entering (just-arrived) object vs. a settled battlefield object — STOP and
     verify against the death-path usage (death also feeds `matches_filter` a post-zone-
     change object, so this is already proven safe, but confirm).
  3. If `EffectTarget::TriggeringCreature` does not resolve correctly for The Great Henge's
     ETB trigger (i.e. `ctx.triggering_creature_id` is not populated from
     `entering_object_id` for this trigger class) — flag it; Great Henge's counter fix would
     then need a different `EffectTarget`. (Miirym already uses `TriggeringCreature` on the
     same trigger class via `CreateTokenCopy { source: TriggeringCreature }`, so this path
     is believed proven — but confirm during implementation.)
