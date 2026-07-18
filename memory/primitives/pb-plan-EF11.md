# Primitive Batch Plan: PB-EF11 — low-yield singletons (WheelDraw greatest-discarded + spell-only single-target requirement)

**Generated**: 2026-07-18
**Primitive**: TWO independent DSL capabilities, bundled to amortize PB overhead, shipped as
**two cleanly-separated commits**:
- **F1 (EF-W-MISS-8)**: `WheelDraw::GreatestDiscarded` — a wheel-draw count = the greatest
  number of cards any affected player discarded this way. Unblocks **Windfall**.
- **F2 (EF-W-MISS-9)**: `TargetRequirement::TargetSpellWithSingleTarget` — a spell-ONLY
  single-target restriction (the existing `TargetSpellOrAbilityWithSingleTarget` is
  over-permissive; it legalizes abilities). Unblocks **Misdirection**.
**CR Rules**: 121.1 (draw), 701.9/701.24 (wheel disposal), 115.7a/115.7b (change target),
115.10 (targeting), 601.2c (target legality), 118.9 (pitch/alternative cost).
**Cards affected**: 2 new (windfall.rs, misdirection.rs). 0 existing fixes.
**Dependencies**: PB-AC9 (`Effect::WheelHand` / `WheelDraw` / `WheelDisposal`) — present.
PB-J (`TargetSpellOrAbilityWithSingleTarget` + validation scaffolding) — present.
PB-AC5 (Pitch alt cost) — present. All confirmed in-tree.
**Deferred items from prior PBs**: none applicable to this batch.

**TODO sweep (roster-recall gate)**: grep of `crates/card-defs/src/defs/` for TODO
comments naming either primitive. Results below (§Card Definition Fixes). **Net: 0 forced
adds beyond the two briefed cards** — the two candidate files do not yet exist (never
authored), so there is no partial/TODO def to flip; both are fresh authors. Confirmed via
`Glob crates/card-defs/src/defs/{windfall,misdirection}.rs` → no files found.

---

## Primitive Specification

### F1 — `WheelDraw::GreatestDiscarded`

`WheelDraw` (`crates/card-types/src/cards/card_definition.rs:2462`) currently has
`ThatMany` (per-player pre-disposal hand size) and `Fixed(u32)`. Windfall needs "draws
cards equal to the greatest number of cards a player discarded this way" — a value shared
across all affected players and only knowable after every player has disposed. Add a third
variant `GreatestDiscarded`.

The existing executor (`crates/engine/src/effects/mod.rs:675`) is a single per-player loop
doing disposal-then-draw. That structure cannot express `GreatestDiscarded`, whose draw
count is a MAX over all players' discard counts and therefore requires all disposals to
complete before ANY draw. The fix is a **two-pass branch keyed on the draw variant**, so
`ThatMany`/`Fixed` retain byte-identical behavior.

### F2 — `TargetRequirement::TargetSpellWithSingleTarget`

`TargetRequirement` (`card_definition.rs:2841`) has `TargetSpellOrAbilityWithSingleTarget`
(discriminant 16) whose validation (`casting.rs:6166-6193`) checks only zone==Stack,
self-target prevention, and `targets.len()==1` — it does NOT check the stack object's kind,
so it legalizes activated/loyalty abilities. Misdirection's oracle is "target **spell** with
a single target" — spell-only. Add `TargetSpellWithSingleTarget` (discriminant 19) that
additionally requires the target stack object be a **spell** (`StackObjectKind::Spell { .. }`
or `StackObjectKind::MutatingCreatureSpell { .. }` — both are cast as spells, CR 601 /
702.140).

The effect reuses the existing `Effect::ChangeTargets { target: DeclaredTarget{0},
must_change: true }` (Bolt Bend precedent, `bolt_bend.rs`; executor `effects/mod.rs:6194`).
`must_change: true` gives the CR 115.7b/ruling-2004-10-04 behavior "if there is no other
legal target, the target is unchanged" (executor's fallback: unchanged when no alternative).

---

## CR Rule Text (from MCP)

- **121.1** — "A player draws a card by putting the top card of their library into their
  hand. This is done as a turn-based action during each player's draw step. It may also be
  done as part of a cost or effect of a spell or ability."
- **115.7a** — "If an effect allows a player to 'change the target(s)' of a spell or
  ability, each target can be changed only to another legal target. If a target can't be
  changed to another legal target, the original target is unchanged... If all the targets
  aren't changed to other legal targets, none of them are changed."
- **115.7b** — "If an effect allows a player to 'change a target' of a spell or ability, the
  process described in rule 115.7a is followed, except that only one of those targets may be
  changed." (Misdirection changes *the* single target; single-target-count guarantees
  115.7a and 115.7b coincide.)
- **115.10 / 115.10a** — being affected does not make an object a target; only "target"
  wording targets.
- **Windfall oracle** (MCP): "Each player discards their hand, then draws cards equal to the
  greatest number of cards a player discarded this way." Sorcery {2}{U}, color identity U.
- **Misdirection oracle** (MCP): "You may exile a blue card from your hand rather than pay
  this spell's mana cost. / Change the target of target spell with a single target." Instant
  {3}{U}{U}, color identity U. Key rulings: "This does not check if the current target is
  legal. It just checks if the spell has a single target." / "If there is no other legal
  target for the spell, this does not change the target." / "You can't make a spell which is
  on the stack target itself."

---

## COMMIT 1 — Feature 1: `WheelDraw::GreatestDiscarded` + Windfall

### Change 1.1: Add the enum variant

**File**: `crates/card-types/src/cards/card_definition.rs`
**Action**: In `enum WheelDraw` (line 2462), add after `Fixed(u32)`:
```rust
/// CR 121.1: draw a number equal to the GREATEST number of cards any affected
/// player disposed of this way (each player's pre-disposal hand size). A shared
/// value across all affected players; computable only after every player has
/// disposed. (Windfall.)
GreatestDiscarded,
```
**Pattern**: Follow the existing `ThatMany`/`Fixed` doc-comment style.

### Change 1.2: Two-pass executor branch

**File**: `crates/engine/src/effects/mod.rs`
**Action**: Restructure the `Effect::WheelHand` arm (lines 675-710). Keep the existing
per-player disposal+draw loop for `ThatMany`/`Fixed` **byte-identical**, but split on the
draw variant at the top of the arm:

- If `draw == WheelDraw::GreatestDiscarded`: two-pass over the resolved player list
  (`resolve_player_target_list` — preserves APNAP for `EachPlayer`):
  1. **Pass 1 (dispose)**: for each player `p`, snapshot `hand_size` (count objects with
     `obj.zone == ZoneId::Hand(p)`) into a per-player `counts` vec, then dispose via the
     same `match disposal { … }` block as today (`discard_cards` / `move_zone_all_then_shuffle`).
  2. Compute `let max_draw = counts.iter().copied().max().unwrap_or(0);`.
  3. **Pass 2 (draw)**: for each player `p` in the same order, draw `max_draw` cards via
     `draw_one_card`, extending `events`.
- Else (`ThatMany` | `Fixed`): the existing single loop, unchanged. Inside it the `match
  draw` at line 702 must add an unreachable-in-this-branch arm — cleanest is to move the
  variant split OUTSIDE the loop so the inner `match draw` only ever sees `ThatMany`/`Fixed`;
  give it `WheelDraw::GreatestDiscarded => unreachable!("handled in two-pass branch above")`
  or restructure so the inner match is not reached. **Prefer** an outer
  `match draw { GreatestDiscarded => { /* two-pass */ } _ => { /* existing loop */ } }`
  wrapper so the existing inner `match draw` keeps only its two arms plus a
  `GreatestDiscarded => unreachable!()` (Rust exhaustiveness will force it).

**CR**: 121.1 (draw), 701.9/701.24 (disposal). The "count = pre-disposal hand size" is per
Windfall's "cards a player discarded this way"; a player with an empty hand discards 0 and
contributes 0 to the max.

**Verify full chain (feedback_verify_full_chain)**: the new variant must be REACHED — the
outer branch dispatches on `draw`, Pass 1 records counts, Pass 2 draws max. The unit test
below asserts every player draws the max even when their own discard was smaller, which
fails if the code accidentally uses per-player counts.

### Change 1.3: Hash arm

**File**: `crates/engine/src/state/hash.rs`
**Action**: In `impl HashInto for WheelDraw` (line 6720), add:
```rust
WheelDraw::GreatestDiscarded => 2u8.hash_into(hasher),
```
Append-only discriminant 2; existing `ThatMany`(0)/`Fixed`(1) output unchanged.

### Change 1.4: Author `windfall.rs`

**File**: `crates/card-defs/src/defs/windfall.rs` (NEW — `build.rs` auto-discovers; no
module registration needed, confirmed by the SR-6 include!/#[path] discovery mechanism).
**Oracle**: "Each player discards their hand, then draws cards equal to the greatest number
of cards a player discarded this way."
**Sketch** (follow `tolarian_winds.rs` / `wheel_of_fortune.rs` shape):
```rust
use crate::cards::helpers::*;
pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("windfall"),
        name: "Windfall".to_string(),
        mana_cost: Some(ManaCost { generic: 2, blue: 1, ..Default::default() }),
        types: types(&[CardType::Sorcery]),
        oracle_text: "Each player discards their hand, then draws cards equal to the \
                      greatest number of cards a player discarded this way.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::WheelHand {
                player: PlayerTarget::EachPlayer,
                disposal: WheelDisposal::Discard,
                draw: WheelDraw::GreatestDiscarded,
            },
            targets: vec![],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}
```
(Confirm the exact `AbilityDefinition::Spell` field set and `PlayerTarget::EachPlayer`
spelling against `tolarian_winds.rs`; this is the byte-for-byte reference.) `completeness`
defaults to `Complete` — correct, this def registers real behavior.

### Change 1.5: Wire bumps for Commit 1

Adding `WheelDraw::GreatestDiscarded`:
- moves the **hash stream fingerprint** (new `HashInto for WheelDraw` arm) AND the **hash
  declared-shape fingerprint** (`WheelDraw` gains a variant; it is in the hashed closure via
  `Effect::WheelHand` → `Effect` → `Characteristics`/`AbilityDefinition`, all hashed in
  `GameState`).
- moves the **protocol schema fingerprint** (`WheelDraw` is reachable on the wire via
  `Effect::WheelHand` → `Effect` → the wire closure; PB-EF10's note confirms
  `Effect`/`AbilityDefinition` are in the closure).

Therefore Commit 1 REQUIRES:
1. `HASH_SCHEMA_VERSION` **53 → 54** (`state/hash.rs:482`); add a `- 54:` History doc line
   describing the `WheelDraw::GreatestDiscarded` arm; append a `HASH_SCHEMA_HISTORY` row with
   the recomputed decl+stream fingerprints; update the `hash_schema_version_sentinel`
   (`tests/core/hash_schema.rs:1193`, currently `53`) and any other `HASH_SCHEMA_VERSION`
   sentinel arms the suite carries.
2. `PROTOCOL_VERSION` **15 → 16** (`rules/protocol.rs:152`); add a `/// - 16:` History doc
   line; set `PROTOCOL_SCHEMA_FINGERPRINT` to the recomputed digest and append a
   `PROTOCOL_HISTORY` row; update the `PROTOCOL_VERSION` sentinel(s) in
   `tests/core/protocol_schema.rs`.

**How to recompute**: run the failing `tests/core/hash_schema.rs` and
`tests/core/protocol_schema.rs` — each prints the freshly computed digest; paste it into the
constant + history row. Do NOT hand-edit a digest.

---

## COMMIT 2 — Feature 2: spell-only single-target `TargetRequirement` + Misdirection

### Change 2.1: Add the enum variant

**File**: `crates/card-types/src/cards/card_definition.rs`
**Action**: In `enum TargetRequirement` (line 2841), add after
`TargetSpellOrAbilityWithSingleTarget` (line 2879):
```rust
/// "target spell with a single target" (CR 115.7a/115.7b, spell-only).
///
/// Like `TargetSpellOrAbilityWithSingleTarget` but the target stack object must be
/// a SPELL (`StackObjectKind::Spell`/`MutatingCreatureSpell`), not an activated or
/// loyalty ability. Validates zone==Stack, kind==spell, exactly one declared target,
/// and applies the same self-target prevention (CR 115.10 ruling: a spell can't be
/// made to target itself). Used by Misdirection.
TargetSpellWithSingleTarget,
```

### Change 2.2: Validation — early-return arm

**File**: `crates/engine/src/rules/casting.rs`, `validate_object_satisfies_requirement`
(lines 6126-6193).
**Action**: Add a new early-return block after the existing
`TargetSpellOrAbilityWithSingleTarget` block (ends line 6194). Mirror that block but add a
**kind check**:
```rust
// CR 115.7a/115.7b: "target spell with a single target" — spell-only.
if matches!(req, TargetRequirement::TargetSpellWithSingleTarget) {
    if obj.zone != ZoneId::Stack {
        return Err(GameStateError::InvalidTarget(format!("object {:?} is not on the stack", id)));
    }
    if let Some(self_oid) = self_id {
        if id == self_oid {
            return Err(GameStateError::InvalidTarget(format!(
                "spell {:?} cannot target itself (self-targeting prevention)", id)));
        }
    }
    let stack_obj = state.stack_objects.iter().find(|so| so.id == id);
    // Spell-only: reject activated/loyalty abilities and non-spell stack objects.
    let is_spell = stack_obj.is_some_and(|so| matches!(
        so.kind,
        StackObjectKind::Spell { .. } | StackObjectKind::MutatingCreatureSpell { .. }
    ));
    if !is_spell {
        return Err(GameStateError::InvalidTarget(format!(
            "stack object {:?} is not a spell (TargetSpellWithSingleTarget is spell-only)", id)));
    }
    let target_count = stack_obj.map(|so| so.targets.len()).unwrap_or(0);
    if target_count != 1 {
        return Err(GameStateError::InvalidTarget(format!(
            "stack object {:?} has {} targets, need exactly 1 for TargetSpellWithSingleTarget",
            id, target_count)));
    }
    return Ok(());
}
```
Confirm `StackObjectKind` is in scope in this file (it is — used at casting.rs:6889); add the
import path if the runner sees an unresolved-name error (`crate::state::stack::StackObjectKind`).

**CR**: 115.7a/115.7b (change target — exactly one), 601.2c (legality), 115.10 (self-target
prohibition per ruling). MutatingCreatureSpell is a spell (CR 702.140) with a single mutate
target — includable; excluding it would wrongly reject a legal Misdirection target.

### Change 2.3: Exhaustive match arms (the #1 compile-error source)

Adding a `TargetRequirement` variant forces new arms in every exhaustive match. Confirmed
sites (matches WITHOUT a `_` catch-all):

| File | Match / fn | Line | Action |
|------|-----------|------|--------|
| `crates/engine/src/state/hash.rs` | `impl HashInto for TargetRequirement` | 5135-5182 | Add `TargetRequirement::TargetSpellWithSingleTarget => 19u8.hash_into(hasher),` (discriminant 19, after `TargetOpponent`=18) |
| `crates/engine/src/rules/casting.rs` | `validate_object_satisfies_requirement`, main `let valid = match req` | 6205-6392 | Add `TargetRequirement::TargetSpellWithSingleTarget => false,` (handled above via early return; mirror line 6386) |
| `crates/engine/src/rules/abilities.rs` | inner battlefield-object `match req` | 7258-7362 | Add `TargetRequirement::TargetSpellWithSingleTarget => false,` (spell targets not applicable to triggered-ability battlefield auto-targeting; mirror line 7358) |

Sites that already have a `_` catch-all and need NO change (verified):
- `crates/engine/src/rules/casting.rs:6096` `validate_player_satisfies_requirement` — `_ =>`
  Err (a spell target is not a player; correct fall-through).
- `crates/engine/src/rules/abilities.rs:7076`/`7203` player/graveyard auto-target match —
  `_ => { battlefield scan }` catch-all.
- `crates/engine/src/rules/abilities.rs:7362` UpToN inner match — has `_ => None`.
- `crates/simulator/**` — grep for `TargetRequirement::` returned NO files; no site.

**After Change 2.3, run `cargo build --workspace`** to surface any match the audit missed
(per infra gotcha: exhaustive matches are the top compile-error source; the replay-viewer /
TUI display matches are on `StackObjectKind`/`KeywordAbility`, NOT `TargetRequirement`, so
they are unaffected — but build the whole workspace to be certain).

### Change 2.4: Author `misdirection.rs`

**File**: `crates/card-defs/src/defs/misdirection.rs` (NEW — auto-discovered).
**Oracle**: "You may exile a blue card from your hand rather than pay this spell's mana cost.
/ Change the target of target spell with a single target."
**Sketch** (Bolt Bend effect + Force of Will pitch, minus PayLife):
```rust
use crate::cards::helpers::*;
pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("misdirection"),
        name: "Misdirection".to_string(),
        mana_cost: Some(ManaCost { generic: 3, blue: 2, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "You may exile a blue card from your hand rather than pay this spell's \
                      mana cost.\nChange the target of target spell with a single target."
            .to_string(),
        abilities: vec![
            // CR 118.9: pitch a blue card instead of the mana cost (no life component).
            AbilityDefinition::AltCastAbility {
                kind: AltCostKind::Pitch,
                cost: ManaCost::default(),
                details: Some(AltCastDetails::Pitch {
                    costs: vec![Cost::ExileFromHand { color: Color::Blue }],
                    opponents_turn_only: false,
                }),
            },
            // CR 115.7a/115.7b: change the target of target spell with a single target.
            AbilityDefinition::Spell {
                effect: Effect::ChangeTargets {
                    target: EffectTarget::DeclaredTarget { index: 0 },
                    must_change: true,
                },
                targets: vec![TargetRequirement::TargetSpellWithSingleTarget],
                modes: None,
                cant_be_countered: false,
            },
        ],
        ..Default::default()
    }
}
```
`completeness` defaults to `Complete`. Verify `Color::Blue`, `AltCostKind::Pitch`,
`AltCastDetails::Pitch`, `Cost::ExileFromHand` against `force_of_will.rs` (all confirmed
present). Note Misdirection has NO cost reduction (unlike Bolt Bend) — omit
`self_cost_reduction`.

### Change 2.5: Wire bumps for Commit 2

Adding `TargetRequirement::TargetSpellWithSingleTarget` moves both fingerprints again
(`TargetRequirement` is in the hashed closure via `AbilityDefinition::targets`, and in the
wire closure via `AbilityDefinition` — PB-EF10 note confirms `AbilityDefinition` reaches the
protocol closure). Therefore Commit 2 REQUIRES:
1. `HASH_SCHEMA_VERSION` **54 → 55**; `- 55:` History line; append `HASH_SCHEMA_HISTORY` row
   with recomputed fingerprints; bump the sentinel to 55.
2. `PROTOCOL_VERSION` **16 → 17**; `/// - 17:` History line; recompute
   `PROTOCOL_SCHEMA_FINGERPRINT` + append `PROTOCOL_HISTORY` row; bump the sentinel.

**Why per-commit bumps (not one shared bump)**: each commit changes the wire/hash type
closure, so for each commit to independently build green (bisectable history) it must carry a
self-consistent version + fingerprint. Commit 1 lands 53→54 / 15→16; Commit 2 lands
54→55 / 16→17. If the coordinator prefers a single combined bump, collapse both into
Commit 2 and leave Commit 1's fingerprints stale — but that makes Commit 1 fail its own
gates, so **per-commit bumps are recommended**.

---

## Unit Tests

### Commit 1 — F1 tests

**File**: `crates/engine/tests/primitives/pb_ef11_wheel_greatest_discarded.rs` (new module;
add `mod pb_ef11_wheel_greatest_discarded;` to `crates/engine/tests/primitives/main.rs`).
**Pattern**: Follow `crates/engine/tests/primitives/pb_ac9_wheel_and_misc.rs` (builds a
multi-player state, sets hand sizes, executes `Effect::WheelHand`, asserts post-draw hand
counts).
- `test_greatest_discarded_all_draw_max` — 3 players with unequal hands (e.g. P0=5, P1=2,
  P2=0). After `WheelHand { EachPlayer, Discard, GreatestDiscarded }`, **every** player's
  hand == 5 (the max). CR 121.1 / Windfall oracle.
- `test_greatest_discarded_decoy_not_per_player` (DECOY, must be non-vacuous) — same setup;
  assert `P1` and `P2` each drew **5**, NOT their own discard counts (2 and 0). This test
  FAILS if the executor uses per-player counts instead of the shared max. (Distinct from the
  `ThatMany` behavior, which pb_ac9 already pins.)
- `test_greatest_discarded_empty_hands` — all players have 0 cards: max is 0, nobody draws;
  no panic (`unwrap_or(0)`).
- `test_greatest_discarded_hash_discriminant` — `hash_of(WheelHand{…GreatestDiscarded})` !=
  `hash_of(WheelHand{…ThatMany})` and `!= …Fixed(5)`; pins discriminant 2 is hashed
  (mirror pb_ac9's hash tests at lines 637-662).
- `test_windfall_card_def` — build a game, cast Windfall via the def, resolve, assert the
  wheel fired (integration using `windfall.rs`).

### Commit 2 — F2 tests

**File**: `crates/engine/tests/primitives/pb_ef11_spell_single_target.rs` (new module; add
to `primitives/main.rs`).
**Pattern**: Follow `crates/engine/tests/rules/copy_redirect.rs:384+`
(`TargetSpellOrAbilityWithSingleTarget` behavioral tests) and `pb_ef6_target_opponent.rs`.
- `test_spell_single_target_accepts_single_target_spell` — put a spell with exactly one
  declared target on the stack; `validate_object_satisfies_requirement(…,
  TargetSpellWithSingleTarget, …)` returns Ok.
- `test_spell_single_target_rejects_two_target_spell` (DECOY, pinned on the count check) — a
  spell with two declared targets is REJECTED. Must fail if the `target_count != 1` guard is
  removed. Ensure the spell IS a `StackObjectKind::Spell` so this test isolates the
  single-target check (distinct from the spell-kind check).
- `test_spell_single_target_rejects_activated_ability` (DECOY, pinned on the kind check) —
  an ActivatedAbility on the stack with exactly one target is REJECTED (spell-only). Must
  fail if the `is_spell` guard is removed. This is the sole difference from
  `TargetSpellOrAbilityWithSingleTarget`.
- `test_spell_single_target_self_prevention` — a spell cannot target itself (pass
  `self_id == id`), Err.
- `test_spell_single_target_hash_discriminant` — `hash_of(TargetSpellWithSingleTarget)` !=
  `hash_of(TargetSpellOrAbilityWithSingleTarget)` (discriminant 19 vs 16). Mirror
  pbt_up_to_n_targets.rs:456.
- `test_misdirection_retargets_single_target_spell` (integration) — P0 casts a single-target
  spell (e.g. a bolt) at creature A (or player). P1 casts Misdirection targeting that spell;
  on resolution `Effect::ChangeTargets` retargets to a different legal object/player.
  Assert the spell's `targets[0]` changed. (Use a scenario where a legal alternative exists;
  the executor picks the smallest-ObjectId alternative in the same zone — known limitation,
  documented at effects/mod.rs:6270 — so ensure the alternative is legal in the scenario.)

**Non-vacuity discipline (per feedback_verify_full_chain)**: each DECOY must fail on EXACTLY
the field under test — verify by locally deleting the guard and confirming the test reddens,
then restore. The two decoys (count check, kind check) are deliberately separated so neither
masks the other.

---

## Verification Checklist

- [ ] `cargo check -p mtg-card-types` (enum variants compile)
- [ ] `cargo build --workspace` (exhaustive matches complete — the seal + match gate)
- [ ] Commit 1: HASH 53→54, PROTOCOL 15→16, both fingerprints recomputed from failing
      schema tests, history rows appended, sentinels updated
- [ ] Commit 2: HASH 54→55, PROTOCOL 16→17, likewise
- [ ] `cargo test --all` (incl. `core hash_schema`, `core protocol_schema`, the two new
      primitives modules, and `core card_defs_fmt`)
- [ ] `cargo clippy --all-targets -- -D warnings`
- [ ] `cargo fmt --check` AND `tools/check-defs-fmt.sh` (SR-35 — the latter is the only one
      that checks windfall.rs / misdirection.rs)
- [ ] windfall.rs and misdirection.rs are `Complete`, register real behavior, no TODOs
- [ ] `python3 tools/authoring-report.py` — coverage delta recorded in collection report
- [ ] Two cleanly-separated commits (F1, then F2)

---

## Risks & Edge Cases

- **`ThatMany`/`Fixed` byte-identity (F1)**: the two-pass restructure must not perturb the
  existing per-player loop. Keep it as the `_` arm of an outer `match draw`; pb_ac9's tests
  are the regression guard — they must stay green untouched.
- **Disposal mode with GreatestDiscarded**: the primitive is general over `WheelDisposal`,
  but the "count" is always the pre-disposal hand size regardless of disposal kind. Windfall
  uses `Discard`. A player with an empty hand contributes 0 (does not raise the max).
- **APNAP (F1)**: all disposal happens before all draws — correct for a single-controller
  sorcery resolution (one resolution, no priority between disposal and draw). Order within
  each pass follows `resolve_player_target_list` (turn order for `EachPlayer`).
- **MutatingCreatureSpell inclusion (F2)**: including it is correct (it is a spell with a
  single mutate target). If the runner finds it complicates the decoy, it may restrict to
  `StackObjectKind::Spell` only and note the deviation — but the plan's recommendation is to
  include it (CR 702.140).
- **ChangeTargets retarget legality (F2)**: the executor's smallest-ObjectId retarget does
  NOT re-check the changed spell's own `TargetRequirement` (documented limitation,
  effects/mod.rs:6270). This is pre-existing (Bolt Bend shares it) and out of scope; the
  integration test must use a scenario where the picked alternative is legal.
- **Double wire bump across two commits**: two sequential fingerprint recomputes. Each commit
  must be independently green; do not defer Commit 1's bump to Commit 2.
- **Fingerprint recompute is machine-driven**: never hand-author a blake3 digest — read it
  from the failing `hash_schema.rs` / `protocol_schema.rs` output.
