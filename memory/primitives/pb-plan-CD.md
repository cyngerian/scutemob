# PB-CD: Counter-doubling replacement effects (CR 122.6 / 614.1)

**Task**: scutemob-18
**Branch**: `feat/pb-cd-counter-doubling-replacement-effects-cr-1216-hardened-`
**Cards in scope**: Hardened Scales, Corpsejack Menace, Conclave Mentor (replacement half only)
**Status**: Plan complete → ready for implementation

---

## 1. CR references

- **CR 122.6** — counter-doubling/extra-counter replacements operate on the boundary "if N
  counters would be put on …" (CR is renumbered; project comments still say 122.6).
  Engine source already cites both interchangeably (e.g. `replacement.rs:2515`).
- **CR 614.1a** — "instead" wording marks a replacement effect, not a trigger.
- **CR 614.5** — a replacement effect applies at most once per event (already enforced by
  `find_applicable` via `already_applied` tracking in the iteration loop).
- **CR 616.1** — multiple applicable replacements: affected player chooses order. M10+
  interactive choice deferred; for now deterministic controller order is used (per existing
  PB-12 pattern). Two-Hardened-Scales ruling ("each adds one") is preserved by
  deterministic iteration; the test plan covers two-Scales stacking.

---

## 2. Oracle text — verified via mcp__mtg-rules__lookup_card

### Hardened Scales ({G}, Enchantment)
> If one or more +1/+1 counters would be put on a creature you control, that many plus
> one +1/+1 counters are put on it instead.

- **Counter type**: +1/+1 ONLY (NOT loyalty, -1/-1, charge, etc.)
- **Receiver scope**: creature you control (NOT planeswalkers, NOT artifacts/enchantments)
- **Modification**: `AddExtraCounter`
- **Rulings of note**:
  - "Each additional Hardened Scales you control will increase the number of +1/+1 counters
    placed on a creature you control by one." (CR 614.5 / two-Scales test target)
  - "If a creature you control would enter the battlefield with a number of +1/+1 counters
    on it, it enters with that many plus one instead." (Already covered by existing
    `apply_counter_replacement` call from `replacement.rs:1424` inside the
    `EntersWithCounters` arm.)

### Corpsejack Menace ({2}{B}{G}, Creature — Fungus 4/4)
> If one or more +1/+1 counters would be put on a creature you control, twice that many
> +1/+1 counters are put on it instead.

- **Counter type**: +1/+1 ONLY
- **Receiver scope**: creature you control
- **Modification**: `DoubleCounters`

### Conclave Mentor ({G}{W}, Creature — Centaur Cleric 2/2)
> If one or more +1/+1 counters would be put on a creature you control, that many plus
> one +1/+1 counters are put on that creature instead.
> When this creature dies, you gain life equal to its power.

- **Replacement half** — same shape as Hardened Scales (AddExtraCounter).
- **Death-trigger half** — **OUT OF SCOPE** for PB-CD. The trigger "you gain life equal
  to its **power**" (NOT toughness as the task brief mistakenly states) requires
  EffectAmount::SourcePower with LKI snapshot at death (CR 603.10a / 113.7a). PB-LKI-CC
  shipped LKI **counters** for SelfDies / SelfLeavesBattlefield; it does NOT thread LKI
  **power** through `PendingTrigger`. The existing card stub already documents this gap
  (`conclave_mentor.rs:18`). PB-CD ships the replacement half; the death trigger remains
  a TODO comment and is filed as a new OOS seed (OOS-LKI-5 / OOS-LKI-Power) for the next
  micro-PB.

  **Rationale for shipping Conclave Mentor anyway**: per planner guidance in the task
  brief — "if death trigger is blocked, ship card-def with TODO on that ability only and
  flag as OOS." The replacement half is the EDH-relevant text in most games; the death
  trigger is a small lifegain rider. Shipping the replacement reduces TODO count by 1
  without committing wrong game state on the replacement (the unshipped trigger is
  documented as blocked, not silently broken).

---

## 3. Infrastructure audit (verified by reading source)

### What already exists

| Component                               | Location                                | Notes                                                                                  |
| --------------------------------------- | --------------------------------------- | -------------------------------------------------------------------------------------- |
| `apply_counter_replacement`             | `replacement.rs:2535`                   | Boundary fn. Takes `_counter: &CounterType` but **does not** filter on it.            |
| `ReplacementTrigger::WouldPlaceCounters`| `replacement_effect.rs:51`              | Two fields: `placer_filter`, `receiver_filter`. **No counter-type field.**             |
| `ReplacementModification::DoubleCounters` | `replacement_effect.rs:151`           | Vorinclex (placer=controller). Verified.                                               |
| `ReplacementModification::AddExtraCounter`| `replacement_effect.rs:159`           | Pir (receiver=ControlledBy). Verified.                                                 |
| `ObjectFilter::AnyCreature`             | `replacement_effect.rs:219`             | Layer-resolved type check (`object_matches_filter:390`). No controller scope.          |
| `ObjectFilter::ControlledBy(PlayerId)`  | `replacement_effect.rs:216`             | Any permanent type (not just creatures). PlayerId(0) bound at registration.            |
| 4 call sites for `apply_counter_replacement` | `replacement.rs:1424`, `effects/mod.rs:1764, 1799, 2963` | All pass `&counter` already. No call-site changes needed for counter-type gating. |
| Vorinclex / Pir / Lae'zel card defs     | `cards/defs/`                           | Existing consumers of `WouldPlaceCounters`. Must continue to work unchanged.           |

### Engine gaps (two items, both confirmed needed)

#### Gap 1 — Counter-type filter on `WouldPlaceCounters`

- **Problem**: Hardened Scales must apply ONLY to +1/+1 counters. With current
  implementation (counter-type ignored), Scales would also double e.g. -1/-1 / loyalty
  counters on a creature you control — clearly wrong.
- **Smallest-surface fix**: add `counter_filter: Option<CounterType>` field to
  `ReplacementTrigger::WouldPlaceCounters`. `None` = any counter type (preserves
  Vorinclex/Pir/Lae'zel behavior — they say "one or more counters" with no type qualifier
  in the engine sense; they do match all counter types intentionally). `Some(t)` = trigger
  only matches when the event's counter type equals `t`.
- **Match logic** in `find_applicable`'s WouldPlaceCounters arm: counter_filter passes if
  effect's filter is `None` OR if effect's filter is `Some(t)` and event's filter is
  `Some(t)` with t equal.
- **Event-side construction**: `apply_counter_replacement` already has the counter type
  in scope. Construct the event trigger with `counter_filter: Some(counter.clone())`.
- **Hashing**: add the field to `state/hash.rs` discriminant 6 (WouldPlaceCounters). Use
  the existing `Option<CounterType>` hashing pattern (option tag + counter type discriminant).
  Bump engine hash version comment.

#### Gap 2 — Receiver filter for "creature you control"

- **Problem**: Current filters are "any creature" (no controller) or "any permanent of
  controller" (no type). Hardened Scales / Corpsejack / Conclave Mentor say "creature
  you control" — both axes simultaneously.
- **Smallest-surface fix**: add `ObjectFilter::CreatureControlledBy(PlayerId)` variant.
  PlayerId(0) placeholder bound at registration (mirrors `ControlledBy(PlayerId(0))`).
  In `object_matches_filter`, use layer-resolved `calculate_characteristics` to read
  card types (mirrors `AnyCreature` pattern at `replacement.rs:390-399`); also check
  `obj.controller == player`.
- **Why a new variant vs composition**: there is no DSL surface for AND-ing two
  `ObjectFilter` variants. Adding a composed/combined variant matches the existing
  enum pattern (Commander, HasCardId, OwnedByOpponentsOf are all special cases too).

### What does NOT need changing

- Call-site count: same 4 sites; no changes to caller signatures.
- `apply_counter_replacement_player` (Vorinclex's TODO branch) — unchanged.
- `ManaWouldBeProduced` / other replacement triggers — orthogonal.
- LKI counter snapshot (PB-LKI-CC shipped, HASH 15) — orthogonal per OOS-LKI-1
  in `memory/primitives/pb-retriage-CC.md:478` (Scales × LKI confirmed no-interaction).
- Existing tests in `counter_replacement.rs` (8 tests for Vorinclex/Pir) — all keep
  passing with `counter_filter: None` default behavior (verified by reading test
  construction patterns at `counter_replacement.rs:37-55`, `77-84`).

---

## 4. Implementation plan (steps)

### Step 1 — engine type changes

1. `crates/engine/src/state/replacement_effect.rs`
   - Add `counter_filter: Option<CounterType>` field to `WouldPlaceCounters` variant.
   - Add `ObjectFilter::CreatureControlledBy(PlayerId)` variant. Docstring per project
     convention (CR ref + use case + binding note).
2. `crates/engine/src/state/hash.rs`
   - Update `WouldPlaceCounters` (disc 6) hashing to include `counter_filter`. Use
     `Option<CounterType>` pattern: `0u8` for `None`, `1u8 + counter.hash_into(...)` for
     `Some`. Mark with comment: "PB-CD adds counter_filter field — bump engine hash."
   - Add `ObjectFilter::CreatureControlledBy(player)` arm at discriminant 8 (next after
     `OwnedByOpponentsOf` = 7).
   - Add PB-CD entry to the hash-version changelog comment at the top of `hash.rs`
     (current HASH 15 → 16).

### Step 2 — `find_applicable` / `event_object_matches_filter` updates

3. `crates/engine/src/rules/replacement.rs`
   - Destructure `counter_filter` in the `WouldPlaceCounters` match arm of `find_applicable`
     (around line 301-313). Add condition: `match_counter_filter(eff_counter, evt_counter)`.
     Implement `match_counter_filter` as a private helper: effect `None` ⇒ true;
     effect `Some(t)` ⇒ event `Some(t)` with equality.
   - Extend `object_matches_filter` (around line 380) with a `CreatureControlledBy(player)`
     arm: layer-resolved type check (mirror `AnyCreature` arm) AND
     `obj.controller == *player`.
   - Extend `bind_object_filter` (around line 469) to bind
     `CreatureControlledBy(PlayerId(0))` → `CreatureControlledBy(controller)`.
   - Update `apply_counter_replacement` (around line 2546) to construct event trigger
     with `counter_filter: Some(counter.clone())`.
   - Update `register_permanent_replacements`'s pattern-match on `WouldPlaceCounters`
     (line 1673-1679) to thread `counter_filter` through unchanged.

### Step 3 — sweep existing constructions

4. Add `counter_filter: None` to every existing `ReplacementTrigger::WouldPlaceCounters`
   construction. Verified call sites:
   - `cards/defs/vorinclex_monstrous_raider.rs:26, 36` (2 sites)
   - `cards/defs/pir_imaginative_rascal.rs:32` (1 site)
   - `cards/defs/laezel_vlaakiths_champion.rs:27` (1 site)
   - `tests/counter_replacement.rs:38, 48, 77` (3 sites — test fixtures)

### Step 4 — card definitions

5. Fill in `crates/engine/src/cards/defs/hardened_scales.rs`:
   - `AbilityDefinition::Replacement` with
     - `trigger: WouldPlaceCounters { placer_filter: Any, receiver_filter: CreatureControlledBy(PlayerId(0)), counter_filter: Some(PlusOnePlusOne) }`
     - `modification: AddExtraCounter`
     - `is_self: false, unless_condition: None`
6. Fill in `crates/engine/src/cards/defs/corpsejack_menace.rs`:
   - Same shape, `modification: DoubleCounters`.
7. Update `crates/engine/src/cards/defs/conclave_mentor.rs`:
   - Add the replacement ability (same shape as Hardened Scales).
   - Keep the existing TODO comment for the death trigger, refresh wording to point
     at OOS-LKI-Power seed (see Step 6).

### Step 5 — tests

8. Add to `crates/engine/tests/counter_replacement.rs` (or new
   `tests/hardened_scales.rs` — choose existing file to avoid sprawl):
   - **Positive — Hardened Scales**: +1/+1 counter placed → +1 added (3 → 4).
   - **Positive — Corpsejack Menace**: +1/+1 counter placed → doubled (3 → 6).
   - **Positive — Conclave Mentor**: +1/+1 counter placed → +1 added (3 → 4).
   - **Isolation — Hardened Scales × counter type**: -1/-1 counter on creature you
     control → unchanged (3 → 3). Loyalty counter on a "creature" you control →
     unchanged (3 → 3). +1/+1 on opponent's creature → unchanged (3 → 3).
   - **Isolation — Hardened Scales × type**: +1/+1 counter on a non-creature permanent
     you control (e.g., an Enchantment with `card_id`) → unchanged (3 → 3). This
     catches the "creature, not any permanent" half of Gap 2.
   - **Stacking — two Hardened Scales**: +1/+1 placed → +2 added (3 → 5). Per ruling
     and CR 614.5.
   - **Stacking — Hardened Scales × Corpsejack Menace**: +1/+1 placed → Corpsejack
     doubles (6), Scales adds 1 (7). Order is deterministic per existing implementation.

### Step 6 — OOS seed entry

9. Append to `memory/primitives/pb-retriage-CC.md` an `OOS-LKI-5` (or `OOS-LKI-Power`)
   entry covering Conclave Mentor's death trigger + Juri Master's death trigger
   (`cards/defs/juri_master_of_the_revue.rs:37` documents the same gap). Pattern:
   "When this dies, [effect] equal to its power" needs LKI power snapshot on
   `PendingTrigger.lki_power: Option<i32>` or equivalent — orthogonal to PB-LKI-CC
   which only ships counters.

### Step 7 — build/test/review

10. `cargo build --workspace`, `cargo test --all`, `cargo clippy --workspace
    --all-targets -- -D warnings`, `cargo fmt --check`.
11. Run `primitive-impl-reviewer` agent. Fix HIGH/MEDIUM findings in-task.
12. Update CLAUDE.md Current State: test count delta, add PB-CD to the PB list,
    refresh next-session pointers, HASH 15 → 16.
13. Regenerate authoring report if any new cards entered.

---

## 5. Risk and stop-and-flag posture

- **Stop and flag** (do NOT silently expand scope) if any of these turn up during
  implementation, per task brief:
  - Counter-type filter requires a new `ObjectFilter` variant beyond
    `CreatureControlledBy` — STOP. Plan only covers one new variant; anything more
    means the brief's "verify existing ObjectFilter" assumption was wrong and the
    scope should be re-triaged.
  - LKI counter snapshot is needed at this boundary — STOP. Per OOS-LKI-1 it is
    confirmed orthogonal, but if implementation surfaces a counter-example, STOP.
  - Multiple Scales interaction (CR 614.5) requires changes to `find_applicable`'s
    `already_applied` tracking — STOP. The current tracking is per-call (already
    in place since PB-12); the test plan validates it works without changes.

- **Yield calibration** (per feedback memo): 3 named cards. If Conclave Mentor's
  death trigger turns out trivially fixable (LKI power infrastructure secretly
  exists), ship it; otherwise file OOS-LKI-Power and ship the replacement half.
  No expansion beyond the three brief-named cards in this PB.

---

## 6. Hash version change

- HASH 15 → HASH 16 — new fields on `WouldPlaceCounters` and new variant on `ObjectFilter`.
- Comment in `hash.rs` at the version-changelog block (top of file).

---

## 7. Acceptance-criteria mapping

| ID   | Description                                          | Step(s)         |
| ---- | ---------------------------------------------------- | --------------- |
| 3754 | Plan file written                                    | This document   |
| 3755 | Counter-type gating + isolation test                 | Step 1, 2, 5    |
| 3756 | Receiver filter for "creature you control"           | Step 1, 2       |
| 3757 | Card definitions (Hardened Scales + Corpsejack)      | Step 4          |
| 3758 | Positive + isolation tests, all tests pass           | Step 5, 7       |
| 3759 | cargo build/test/clippy/fmt clean                    | Step 7          |
| 3760 | primitive-impl-reviewer pass + HIGH/MED fixes        | Step 7          |
| 3761 | CLAUDE.md refresh + authoring report                 | Step 7          |

---

## 8. Estimated diff size

- **State types**: ~30 lines added (2 field/variant adds + hashing + match arms).
- **Rules logic**: ~25 lines added (match arms in `find_applicable`,
  `object_matches_filter`, `bind_object_filter`, call-site change in
  `apply_counter_replacement` and registration).
- **Existing call sites swept**: 7 sites, +1 line each (`counter_filter: None`).
- **Card defs**: 3 files, ~15 lines each.
- **Tests**: ~8 tests, ~200 lines total.
- **Total**: ~350 lines diff. Single coherent PB — within bulk-card-author / primitive-impl-runner one-shot range.
