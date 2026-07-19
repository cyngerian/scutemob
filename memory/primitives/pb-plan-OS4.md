# Primitive Batch Plan: PB-OS4 — return-transformed / enters-transformed as a NEW object (OOS-EF5-3)

**Generated**: 2026-07-19
**Primitive**: A permanent that is **exiled (or dies) and returns to the battlefield already on its
back face** — a **NEW object** entering transformed (CR 400.7 + CR 712.18). This is a fundamentally
different mechanism from PB-EF5's in-place `Effect::TransformSelf` (which keeps the SAME `ObjectId`,
CR 712.18-for-transform). Two new unit `Effect` variants (immediate + delayed) + one new
`DelayedTriggerAction` variant, threaded through the existing zone-move / delayed-return machinery.
**CR Rules**: 400.7 (new object on zone change), 712.18 (which face; transform-in-place vs a new
object entering transformed), 603.7 (delayed triggered abilities — "at the next end step"), 714
(Sagas — chapter abilities, final-chapter sacrifice, and how Fable ch. III's exile-then-return
replaces the usual sacrifice), 306.5b (planeswalker starting loyalty — the blocker for the
planeswalker-back cards).
**Cards affected**: 4 candidates verified vs oracle/rulings; **honest ship ~2 flips**
(`fable_of_the_mirror_breaker`, `edgar_charmed_groom`) + **2 stay out** with NAMED blockers
(`nicol_bolas_the_ravager`, `grist_voracious_larva` — planeswalker-back starting-loyalty gap).
**Dependencies**: PB-EF5 (`Effect::TransformSelf`, back-face layer resolution — shipped). Reuses
`Effect::MoveZone`, `Effect::CreateTokenCopy`, `Effect::ExileWithDelayedReturn`, `DelayedTrigger`
machinery, `SagaChapter` ability, `check_saga_sbas` (CR 714.4).
**Deferred items from prior PBs**: OOS-EF5-3 itself (filed by PB-EF5). No other PB points here.
**Wire bump**: **PROTOCOL 18 → 19, HASH 55 → 56** (machine-forced by SR-8; justified in §4).

---

## 0. TODO sweep (roster-recall gate — MANDATORY, per feedback_planner_roster_recall)

Ran over `crates/card-defs/src/defs/`:
```
Grep -i "TODO.*[Tt]ransform | return.*transform | enters.*transform | new object.*transform
        | return.*battlefield.*transform"
```
**Result: 0 cards with matching comments.** None of the 4 candidate cards has a def file (all
`missing` — confirmed by Glob: `edgar_charmed_groom.rs`, `fable_of_the_mirror_breaker.rs`,
`nicol_bolas_the_ravager.rs`, `grist_voracious_larva.rs` do **not** exist). PB-EF5's own TODO sweep
(pb-plan-EF5 §0) already established these four have no def file — they were explicitly *not*
authored by PB-EF5 (§6 verdict "do not author — OOS-EF5-3"). **TODO sweep: 0 forced adds.**

Roster is therefore exactly the 4 candidates from the brief; no hidden self-identified cards.

---

## 1. Primitive Specification

### The mechanism (why it is NOT `TransformSelf`)
`Effect::TransformSelf` (PB-EF5) flips a DFC **in place**: same `ObjectId`, counters/damage/Auras
persist (CR 712.18). **Return-transformed is the opposite**: the permanent leaves the battlefield
(to graveyard or exile) — becoming a new object per CR 400.7 — and a *further* new object is put
onto the battlefield **already showing its back face**. Consequences that the tests must pin:
- **New `ObjectId`** (CR 400.7): the pre-departure id is dead.
- **Counters do NOT carry** (fresh object, default `counters`).
- **Auras/Equipment do NOT carry** (they fall off as the permanent leaves; SBA 704.5m/704.5n).
- **"When this dies" triggers reference the OLD object** (LKI at death), not the returned one
  (CR 603.7c: "if that object left that zone and then returned, it's a new object").
- The returned object's characteristics are the **back face**, **layer-resolved** through
  `calculate_characteristics` (layers.rs:97 already substitutes `back_face` when `is_transformed`).

### Two timing modes (both in this PB)
1. **Immediate** (fable ch. III, nicol_bolas, grist): during one resolution, exile the source then
   return it transformed. → new unit `Effect::ExileSourceAndReturnTransformed`.
2. **Delayed at next end step** (edgar): the source *dies* (a WhenDies trigger), and at the beginning
   of the next end step it returns from the graveyard transformed (CR 603.7). → new unit
   `Effect::ReturnSourceToBattlefieldTransformedNextEndStep`, which registers a `DelayedTrigger`
   carrying a new `DelayedTriggerAction::ReturnFromGraveyardToBattlefieldTransformed`.

### Design decision: dedicated effects, NOT a flag on `Effect::MoveZone`
The retriage brief allowed "a flag on `Effect::MoveZone` **or** a dedicated `Effect::ReturnTransformed`."
**Choose the dedicated effects.** Rationale:
- Adding an `enters_transformed: bool` field to the `Effect::MoveZone` **struct variant** would force
  a field into **every** `Effect::MoveZone { … }` construction site across the card-def corpus (a
  large mechanical blast radius, `..Default` is not used at those literals). A dedicated unit variant
  touches only new sites.
- The oracle phrasing is literally "exile X, then return it transformed" — a single self-contained
  effect models it atomically and handles the CR 400.7j "the effect can find the object it just
  exiled" internally, avoiding fragile `ctx.source`-remap threading across a two-element `Sequence`.
- Matches the PB-EF5 precedent (`TransformSelf` is a unit variant targeting `ctx.source`).

Both new effects target `ctx.source` implicitly (all four cards self-return; for fable the SagaChapter
ability's source is the Saga permanent). No target field needed — mirrors `TransformSelf`,
`SetReturnToHandAtEndStep`.

### Prior art to reuse (do NOT duplicate)
- **`craft` return-transformed** (`engine.rs:1422-1441`): the canonical "move an exiled DFC to the
  battlefield and set `is_transformed = true` + `last_transform_timestamp`" block. The immediate
  executor mirrors this exactly.
- **`DelayedTriggerAction::ReturnFromExileToBattlefield { tapped }`** dispatch
  (`resolution.rs:7469-7520`): registers statics, queues ETB triggers, emits
  `PermanentEnteredBattlefield`. The new graveyard-return dispatch mirrors it (from graveyard, and
  sets `is_transformed`).
- **`Effect::ExileWithDelayedReturn`** executor (`effects/mod.rs:5390-5449`): the
  `state.delayed_triggers.push_back(DelayedTrigger { … timing: AtNextEndStep … })` idiom the edgar
  effect reuses (edgar's object is already in the graveyard, so it does *not* exile first — it just
  schedules the return).
- **`SagaChapter { chapter, effect }`** (`card_definition.rs:490`): the chapter's `effect` is an
  arbitrary `Effect`, so Fable ch. III is simply `SagaChapter { chapter: 3, effect:
  ExileSourceAndReturnTransformed }`. No Saga-subsystem change needed.

---

## 2. CR Rule Text (from MCP)

**400.7** — "An object that moves from one zone to another becomes a new object with no memory of, or
relation to, its previous existence." (Exceptions 400.7a-m do not cover a permanent returning to the
battlefield — the returned permanent is unambiguously new.) **400.7j** — "If the cost of a spell or
ability causes an object to move to a public zone, that spell or ability's effects can find that
object." (Lets the single effect find the object it just exiled, to return it.)

**712.18** — "When a double-faced permanent **transforms or converts**, it doesn't become a new
object. Any effects that applied to that permanent will continue to apply to it." (This governs
`TransformSelf`. Return-transformed is NOT a transform of an existing permanent — it is a **new
permanent entering the battlefield transformed**, so 712.18 does not apply; 400.7 does.)

**603.7 / 603.7c** — delayed triggered abilities. 603.7c: "A delayed triggered ability that refers to
a particular object still affects it … However, if that object is no longer in the zone it's expected
to be in … the ability won't affect it. (Note that if that object left that zone and then returned,
it's a new object and thus won't be affected. See rule 400.7.)"

**714.4** — "If the number of lore counters on a Saga permanent … is greater than or equal to its
final chapter number, and it isn't the source of a chapter ability that has triggered but not yet
left the stack, that Saga's controller sacrifices it." (For Fable: after ch. III resolves, the Saga
has been exiled and a NON-Saga creature returned — 714.4 finds no Saga to sacrifice.)

**306.5b** — a planeswalker enters with loyalty counters equal to its printed loyalty. (The blocker
for `nicol_bolas` / `grist`: their back faces are planeswalkers; `CardFace` has no
`starting_loyalty` and the enters-transformed path assigns none → a 0-loyalty planeswalker dies to
SBA 704.5i on entry. See §6.)

### DFC-copy edge (from rulings, all four cards)
"If you are instructed to put a card that isn't a double-faced card onto the battlefield transformed,
it will not enter the battlefield at all. In that case, it stays in the zone it was previously in."
→ the executor must **not** return a non-DFC to the battlefield (it stays in exile / graveyard).
Only reachable via a copy effect (e.g. Mirror Image copying Nicol Bolas); no roster card hits it
without a copy, but the guard is cheap and CR-correct — include it (mirror the craft `is_dfc` guard).

---

## 3. Engine Changes

### Change 1 — `Effect::ExileSourceAndReturnTransformed` variant (DSL, immediate path)
**File**: `crates/card-types/src/cards/card_definition.rs`
**Action**: add a **unit** variant to `enum Effect`, near `Effect::TransformSelf`. Doc-comment with
CR 400.7 + 712.18 (new object, NOT in-place) + the non-DFC-stays guard.
**Pattern**: mirror `Effect::TransformSelf` (unit, targets `ctx.source`).
**Wire consequence**: in the SR-8 protocol fingerprint closure (Characteristics→Effect) **and** the
GameState hash closure → forces PROTOCOL + HASH bumps (§4).

### Change 2 — `Effect::ReturnSourceToBattlefieldTransformedNextEndStep` variant (DSL, delayed path)
**File**: `crates/card-types/src/cards/card_definition.rs`
**Action**: add a second **unit** variant near `SetReturnToHandAtEndStep` (`card_definition.rs:2045`).
Doc-comment CR 603.7 (delayed) + 400.7 (new object) — "when this dies, return it to the battlefield
transformed at the beginning of the next end step" (Edgar).
**Pattern**: mirror `SetReturnToHandAtEndStep`, but instead of setting a flag it registers a
`DelayedTrigger` (see Change 5).

### Change 3 — `DelayedTriggerAction::ReturnFromGraveyardToBattlefieldTransformed` variant
**File**: `crates/card-types/src/state/stubs.rs` (the enum lives in card-types, `:33`; SR-6-safe)
**Action**: add a variant to `enum DelayedTriggerAction` (after `ReturnFromGraveyardToHand`, `:40`).
`#[derive(… Hash …)]` already present. Unit variant (target object carried by `DelayedTrigger`).
**Wire consequence**: `DelayedTriggerAction` is reachable from `Effect` (via
`Effect::CreateTokenCopy.delayed_action`) → in the protocol closure; and it derives `Hash` and is
folded into the GameState hash → both bumps (same single bump as Changes 1/2).

### Change 4 — immediate executor arm
**File**: `crates/engine/src/effects/mod.rs`
**Action**: add a match arm in `execute_effect_inner` (near the `Effect::TransformSelf` arm, and the
`Effect::MoveZone` arm at `:2785` for reference). Logic (mirror craft `engine.rs:1422-1441` +
`MoveZone` exile-event shape):
1. Read `ctx.source`'s owner + `card_id`; if not on the battlefield, no-op (CR fizzle).
2. Determine `is_dfc = def.back_face.is_some()`.
3. Move `ctx.source` → `Exile` (`fizzle_move_object_to_zone`); emit `GameEvent::ObjectExiled`
   (with LKI counters/power like the `MoveZone` arm). Update `ctx.source = exile_id` (CR 400.7j).
4. **If `is_dfc`**: move the exiled object → `Battlefield`; on the new object set
   `is_transformed = true`, `last_transform_timestamp = state.timestamp_counter`,
   `timestamp_counter += 1`, `controller = owner`. Register static continuous effects
   (`replacement::register_static_continuous_effects`) and queue ETB triggers
   (`replacement::queue_carddef_etb_triggers`) — copy the block from `resolution.rs:7489-7513`.
   Emit `GameEvent::PermanentEnteredBattlefield`. Update `ctx.source` to the new battlefield id.
5. **If not `is_dfc`**: leave it in exile (CR ruling — do not return). No ETB event.
**CR**: 400.7 (new object), 712.18 (enters transformed → back-face characteristics), 400.7j.
**Note**: does NOT assign planeswalker loyalty (see §6 — that is why nicol/grist stay out).

### Change 5 — delayed executor arm
**File**: `crates/engine/src/effects/mod.rs`
**Action**: add a match arm for `Effect::ReturnSourceToBattlefieldTransformedNextEndStep`. It
registers a delayed trigger targeting the source (which, for a WhenDies trigger, is the graveyard
object — same as `SetReturnToHandAtEndStep` at `:3991`):
```rust
state.delayed_triggers.push_back(DelayedTrigger {
    source: ctx.source,
    controller: ctx.controller,
    target_object: ctx.source,              // the graveyard object
    action: DelayedTriggerAction::ReturnFromGraveyardToBattlefieldTransformed,
    timing: DelayedTriggerTiming::AtNextEndStep,   // CR 603.7 — next end step, any player's
    fired: false,
});
```
**Pattern**: `Effect::ExileWithDelayedReturn` executor (`effects/mod.rs:5439-5446`).
**CR**: 603.7 (delayed trigger fires at the next end step, not immediately).

### Change 6 — delayed-action dispatch arm
**File**: `crates/engine/src/rules/resolution.rs`
**Action**: add a `DelayedTriggerAction::ReturnFromGraveyardToBattlefieldTransformed` arm in the
delayed-trigger dispatch match (the block containing `ReturnFromExileToBattlefield` at `:7469` and
`ReturnFromGraveyardToHand` at `:7543`). Logic = a merge of the two:
1. Confirm `target` is in a `Graveyard(_)` zone (else fizzle — CR 603.7c / 400.7).
2. Check `is_dfc` (back_face present); if not a DFC, do **not** return (stays in graveyard — CR ruling).
3. `expect_move_object_to_zone(target, Battlefield)`; on the new obj set `controller = owner`,
   `is_transformed = true`, `last_transform_timestamp` + bump `timestamp_counter`.
4. `register_static_continuous_effects` + `queue_carddef_etb_triggers` (copy from `:7489-7513`).
5. Emit `GameEvent::PermanentEnteredBattlefield`.
**CR**: 603.7c, 400.7, 712.18.

### Change 7 — hash discriminants for the two new `Effect` variants
**File**: `crates/engine/src/state/hash.rs`
**Action**: in `impl HashInto for Effect`, add two arms. Current **max Effect discriminant = 93**
(`Effect::TransformSelf => 93u8`, `hash.rs:6738`; next below it `SetNoMaximumHandSize = 92`,
`SetReturnToHandAtEndStep = 71`). Assign:
- `Effect::ExileSourceAndReturnTransformed => 94u8.hash_into(hasher)`
- `Effect::ReturnSourceToBattlefieldTransformedNextEndStep => 95u8.hash_into(hasher)`
Verify no higher discriminant exists by scanning the whole `Effect` arm block before committing (the
sentinel-hash + protocol-fingerprint gates will catch a collision/omission regardless).

### Change 8 — hash arms for the new `DelayedTriggerAction` variant (EXHAUSTIVE — #1 compile-error source)
**File**: `crates/engine/src/state/hash.rs`
`DelayedTriggerAction` is matched at **four** sites — every one needs the new arm, or the crate
won't compile:

| File | Match context | Line | Action |
|------|---------------|------|--------|
| `state/hash.rs` | `HashInto for DelayedTriggerAction` | ~2290 | add discriminant arm (next unused byte in this local match) |
| `state/hash.rs` | `TriggerData::DelayedAction` hash path | ~3349 | add arm |
| `state/hash.rs` | `PendingTrigger` data hash path | ~3673 | add arm |
| `state/hash.rs` | `DelayedTrigger` struct hash path | ~6499 | add arm |

(The dispatch match at `resolution.rs:7469` is Change 6; the executor references are Changes 5/6.
There is **no** `DelayedTriggerAction` match in TUI/replay-viewer.)

### Change 9 — exhaustive-match confirmation (NOT expected to touch display code)
New `Effect` variants do **not** touch `tools/tui/src/play/panels/stack_view.rs` or
`tools/replay-viewer/src/view_model.rs` (those match `StackObjectKind`/`KeywordAbility`, not `Effect`
— confirmed for PB-EF5). **Proof obligation**: `cargo build --workspace` is the only thing that
proves no exhaustive `Effect`/`DelayedTriggerAction` match arm was missed. Run it after the impl phase.

---

## 4. Wire-bump checklist (machine-forced — DO NOT fight the gate)

The two new `Effect` variants and the new `DelayedTriggerAction` variant are all inside the SR-8
protocol fingerprint closure (reachable from `Effect`) **and** the GameState hash closure. Both gates
red until re-pinned. There is **no** new serialized/hashed struct field on `GameObject` or on any
existing variant (the design deliberately reuses `is_transformed` / `DelayedTrigger`, which already
exist and are already hashed) — so the blast radius is exactly "three new enum variants."

- [ ] `PROTOCOL_VERSION` **18 → 19** (`rules/protocol.rs:171`).
- [ ] Re-pin `PROTOCOL_SCHEMA_FINGERPRINT` (`rules/protocol.rs:188`) from the failing `protocol_schema`
      test's expected digest; append a `- 19:` History row.
- [ ] `HASH_SCHEMA_VERSION` **55 → 56** (`state/hash.rs:496`); append a `HASH_SCHEMA_HISTORY` epoch row
      with both fingerprints from the failing sentinel-hash test (`state/hash.rs:518` ledger).
- [ ] Bump any hardcoded sentinel hashes the version bump moves (run failing tests; copy expected).
- [ ] `cargo build --workspace` (exhaustive-match proof) + `cargo test --all`.

Let the gates force the numbers; never hand-compute a fingerprint. **Justification for the bump**
(close-out): PB-OS4 adds genuinely new DSL expressiveness (return-transformed as a new object) that
did not previously exist in any form — the wire schema legitimately changed. This is a capability PB;
the bump is expected and correct.

---

## 5. Per-card chain-verification (oracle/rulings, MCP-verified this plan)

> **Verification caveat (record it):** the MCP `lookup_card` tool returns **type line + keywords +
> rulings only** for double-faced cards (no oracle-text field — confirmed for all four here; it *does*
> return oracle text for single-faced cards, e.g. "Grist, the Hunger Tide", which is a **different
> card** from the DFC "Grist, Voracious Larva"). Front/back **oracle text** below is reconstructed
> from the printed cards + the returned rulings. **The runner MUST re-confirm each face's exact
> wording against cards.sqlite / oracle text at authoring time** before flipping any card `Complete`
> (this is the PB-EF5 lesson — grist and bloodline_keeper had mis-filed 2nd blockers).

| card | faces (MCP type line) | return mechanism | 2nd blocker beyond return-transformed | verdict |
|------|----------------------|------------------|---------------------------------------|---------|
| **fable_of_the_mirror_breaker** | Enchantment — Saga // Enchantment Creature — Goblin Shaman | **immediate**, Saga ch. III: exile self, return transformed | back face = Reflection of Kiki-Jiki `{1}{R},{T}`: copy target nonlegendary creature you control, gains haste, sac at next end step → **`Effect::CreateTokenCopy` EXISTS** (`card_definition.rs:2148`, has `gains_haste` + `delayed_action`). Ch. I token w/ restricted mana ability + ch. II loot to verify. | **flip → Complete** (contingent on ch. I/II + "becomes a Goblin" clause being non-stub; else partial) |
| **edgar_charmed_groom** | Legendary Creature — Vampire Noble // Legendary Artifact | **delayed**, WhenDies → next end step return from graveyard transformed | back face = Edgar Markov's Coffin (artifact) — end-step conditional `TransformSelf` (PB-EF5) + Vampire token/anthem clauses; **no planeswalker/loyalty** (artifact back) | **flip → Complete** (contingent on back-face artifact abilities being expressible; else partial) |
| **nicol_bolas_the_ravager** | Legendary Creature — Elder Dragon // Legendary Planeswalker — Bolas | **immediate**, `{4}{U}{B}{R}`: exile self, return transformed (NOTE: **immediate on resolution, NOT "at next end step"** — the brief's "at next end step" is WRONG; confirmed by ruling "won't be exiled or return transformed" if it leaves before the ability resolves) | back face is a **planeswalker** → **starting-loyalty gap** (§6): `CardFace` has no `starting_loyalty`; enters-transformed assigns no loyalty → 0-loyalty PW dies to SBA 704.5i. Plus 3 back-face loyalty abilities to author. | **STAY OUT** — named blocker (loyalty) |
| **grist_voracious_larva** | Legendary Creature — Insect // Legendary Planeswalker — Grist | **immediate**, ETB-style trigger: "Whenever Grist or another creature you control enters, if it entered from your graveyard or you cast it from your graveyard, you may pay {G}. If you do, exile Grist, then return it to the battlefield transformed under its owner's control." | **two** blockers: (1) same planeswalker starting-loyalty gap (§6); (2) an "entered from your graveyard / was cast from your graveyard" trigger condition — **absent** from the DSL | **STAY OUT** — two named blockers |

**Honest discounted ship: ~2 Complete** (fable, edgar), matching the brief's "~2-3." Both exercise a
distinct timing path of the primitive (immediate via Saga chapter; delayed via WhenDies→end-step).
`nicol_bolas` and `grist` stay unauthored (as PB-EF5 left them) with the loyalty gap recorded — do
NOT author them, because a 0-loyalty planeswalker entering and instantly dying to SBA is *wrong game
state* (W6 policy bars it), not a truthful partial.

### Runner discretion
If, at authoring time, fable's ch. I/II or edgar's back-face clauses require a **gated-stub effect**
(`Effect::Choose`, `AddManaChoice`, etc. — barred from `Complete` per §5 dispatch-loop notes), author
that card `partial` with the return clause wired (real primitive corpus usage) and the residual clause
truthfully marked — do **not** force the flip. Prefer 1 honest Complete over 2 stubbed ones.

---

## 6. The planeswalker-back starting-loyalty gap (NAMED blocker — explicitly OUT of scope)

`CardFace` (`card_definition.rs:30`) has `power`/`toughness`/`color_indicator` but **no
`starting_loyalty`** field, and neither the immediate nor the delayed enters-transformed path assigns
loyalty counters. So a DFC whose **back face is a planeswalker** (nicol_bolas → Nicol Bolas, the
Arisen; grist → Grist, the Plague Swarm) would enter transformed with **0 loyalty** and be put into
the graveyard by SBA 704.5i (CR 704.5i / 306.5b) on the very next SBA check. This is the same
"transform-while-creature → 0-loyalty planeswalker dies" case the rulings call out — but here it
occurs on the card's *intended* path, so it is a genuine blocker, not an edge case.

**Fixing it is a separate primitive** (add `CardFace.starting_loyalty: Option<u32>` + assign
`CounterType::Loyalty` counters in both enters-transformed paths, CR 306.5b) **plus** substantial
back-face authoring (3 loyalty abilities each, some complex — e.g. Nicol Bolas the Arisen's "+7 put a
permanent from your hand onto the battlefield"). **Do not fold it into PB-OS4.** Record it as a
follow-up seed **OOS-OS4-1** (see §10) so nicol/grist can be revisited. This keeps PB-OS4's blast
radius to exactly the three new enum variants.

---

## 7. Saga-chapter integration for Fable (CR 714)

No Saga-subsystem code change is required. Model Fable ch. III as:
```
AbilityDefinition::SagaChapter { chapter: 3, effect: Effect::ExileSourceAndReturnTransformed }
```
When the ch. III triggered ability resolves, the executor exiles the Saga (its `ctx.source`) and
returns it to the battlefield transformed as Reflection of Kiki-Jiki (a non-Saga creature). The CR
714.4 sacrifice SBA (`sba.rs:827 check_saga_sbas`) then finds **no** Saga on the battlefield with
lore ≥ final chapter (the returned object has no `SagaChapter` abilities), so it is **not**
sacrificed — the desired result. Verify the ordering test in §8 pins this (the returned creature
survives; the Saga is gone). CR 714.4's "isn't the source of a chapter ability … not yet left the
stack" clause is naturally satisfied because the ch. III ability has fully resolved (and its source
left the battlefield) before the next SBA check.

**Saga edge to verify at authoring**: Fable's back-face Reflection is a **creature** — CR 714.1a
"Saga enchantments that also have the type creature" is *not* Fable (its FRONT is a non-creature
Saga; the back is a creature but not a Saga). No interaction with lore counters on the back face.

---

## 8. Unit Tests

**File**: `crates/engine/tests/mechanics_m_z/pb_os4_return_transformed.rs` (new module; add
`mod pb_os4_return_transformed;` to `crates/engine/tests/mechanics_m_z/main.rs` — SR-9a: a file with
no `mod` line silently does not compile / silently drops coverage). Pattern: follow
`mechanics_m_z/transform.rs`, `pb_ef5_transform_self.rs`, and the delayed-return tests for
`ExileWithDelayedReturn`.

Each test cites CR; **each decoy fails on exactly the field under test** (SR-34/36 probe-by-execution).

**Immediate path (`ExileSourceAndReturnTransformed`):**
- `test_return_transformed_new_object_identity` — a battlefield DFC with an ability resolving
  `ExileSourceAndReturnTransformed`: assert the resulting battlefield object has a **new `ObjectId`**
  (≠ the pre-exile id), `is_transformed == true`. CR 400.7 / 712.18.
- `test_return_transformed_counters_do_not_carry` — put +1/+1 counters on the source, return it,
  assert the new object's `counters` is empty. CR 400.7. **Decoy**: a plain in-place `TransformSelf`
  on the same setup keeps the counters (proves this path is the new-object path, not TransformSelf).
- `test_return_transformed_aura_falls_off` — an Aura enchants the source; after return the Aura is in
  its owner's graveyard (SBA 704.5m) and the new object is unenchanted. CR 400.7 / 704.5m.
- `test_return_transformed_back_face_characteristics` — after return, `calculate_characteristics` of
  the new object returns the **back face's** name/types/P-T (layer-resolved, not the raw front def).
  CR 712.18. **Decoy**: assert the FRONT-face name is NOT what's read.
- `test_return_transformed_non_dfc_stays_in_exile` — a non-DFC source: `ExileSourceAndReturnTransformed`
  exiles it and it does **not** return (stays in exile). CR ruling (put-non-DFC-transformed → doesn't
  enter). **Decoy**: the DFC variant of the same setup DOES return.

**Delayed path (`ReturnSourceToBattlefieldTransformedNextEndStep`):**
- `test_delayed_return_transformed_timing` — a WhenDies trigger schedules the return; assert the object
  is **still in the graveyard** immediately after the trigger resolves, and enters the battlefield
  transformed only **at the next end step**. CR 603.7. **Decoy**: assert it is NOT on the battlefield
  before the end step (a bug that returned immediately would fail this).
- `test_delayed_return_transformed_is_new_object` — the object returned at end step has a new
  `ObjectId` and `is_transformed == true`; a "when this dies" ability keyed to the OLD object does not
  see the returned object. CR 400.7 / 603.7c.

**Saga (Fable):**
- `test_saga_return_transformed_chapter_three_no_sacrifice` — drive a Saga to 3 lore counters, resolve
  the ch. III `ExileSourceAndReturnTransformed`; assert (a) the returned object is the **back-face
  creature** on the battlefield, (b) the CR 714.4 SBA does **NOT** sacrifice it (it is no longer a
  Saga). CR 714.4 / 400.7. **Decoy**: a Saga whose ch. III does a plain effect (not return-transformed)
  IS sacrificed by 714.4 — proves the no-sacrifice result is due to the object leaving+returning.

**Card-def integration (only for flipped cards):**
- `test_fable_transforms_at_chapter_three` — full Fable: ch. I token, ch. II loot, ch. III returns
  transformed to Reflection of Kiki-Jiki, not sacrificed. (Author-time; probe-by-execution.)
- `test_edgar_returns_transformed_at_end_step` — Edgar dies → at next end step returns to the
  battlefield transformed as Edgar Markov's Coffin (artifact), new object, no counters carried.
- `test_nicol_bolas_and_grist_not_complete` — assert `nicol_bolas_the_ravager` / `grist_voracious_larva`
  have **no `Complete` def** (integrity guard: they must not be force-flipped while the loyalty gap
  stands). If left unauthored, assert absence from the registry; if authored `partial`, assert the
  marker names the loyalty blocker.

---

## 9. Verification Checklist

- [ ] `Effect::ExileSourceAndReturnTransformed` + `Effect::ReturnSourceToBattlefieldTransformedNextEndStep`
      added (card-types); executor arms (Changes 4/5) + hash arms 94/95 (Change 7) added.
- [ ] `DelayedTriggerAction::ReturnFromGraveyardToBattlefieldTransformed` added (card-types stubs);
      dispatch arm (Change 6) + all **4** hash arms (Change 8) added.
- [ ] `cargo check` clean.
- [ ] fable flipped Complete (or honest partial); edgar flipped Complete (or honest partial);
      nicol_bolas + grist left out with the loyalty blocker recorded (NOT force-flipped).
- [ ] PROTOCOL 18→19 + `PROTOCOL_SCHEMA_FINGERPRINT` re-pinned + `- 19:` history row; HASH 55→56 +
      epoch row; sentinel hashes updated (all from failing-gate expected output).
- [ ] `cargo build --workspace` (exhaustive-match proof), `cargo test --all`,
      `cargo clippy --all-targets -- -D warnings`, `cargo fmt --check` + `tools/check-defs-fmt.sh`.
- [ ] OOS-EF5-3 closed (SHIPPED banner + table strike) in `oos-retriage-plan-2026-07-18.md` §3 PB-OS4
      and `ef-batch-plan-2026-07-17.md` §9; OOS-OS4-1 (loyalty gap) filed; WIP status updated.
- [ ] No remaining return-transformed TODO in shipped defs; nicol/grist blocker named in the seed.

---

## 10. Risks & Edge Cases

- **Planeswalker-back loyalty gap** (§6) — the reason nicol/grist stay out. **STOP-and-flag if the
  runner is tempted to force-flip them**: a 0-loyalty planeswalker dying to SBA on entry is wrong game
  state, not a truthful partial. File **OOS-OS4-1** (add `CardFace.starting_loyalty` + assign CR
  306.5b loyalty in both enters-transformed paths + author the planeswalker back faces).
- **Back-face ETB triggers on enter-transformed** — the reused `queue_carddef_etb_triggers` /
  `register_static_continuous_effects` use the card def; verify they honor `is_transformed` (fire the
  **back** face's ETBs/statics, not the front's). Neither fable's Reflection nor edgar's Coffin has a
  battlefield-relevant ETB trigger, so this doesn't gate the two flips — but **verify** and, if the
  back-face ETBs don't fire, record it as a limitation (do not silently ship a card whose back-face
  ETB matters).
- **`nicol_bolas` timing correction** — the brief and `oos-retriage-plan` §3 both say nicol_bolas
  returns "at next end step." **That is wrong** (confirmed by ruling): its `{4}{U}{B}{R}` ability
  exiles-and-returns **immediately on resolution**. Recorded here so the runner doesn't wire it to the
  delayed path. (Moot unless the loyalty gap is later fixed, but flag it.)
- **DFC-copy edge** (CR ruling) — a non-DFC copy told to enter transformed must NOT enter (stays in
  exile/graveyard). Both executor paths include the `is_dfc` guard; a test pins the immediate path.
  The copy-effect path itself is not exercised by any roster card — do not build extra machinery for it.
- **Saga 714.4 double-check** — confirm the returned non-Saga creature is not swept by the Saga SBA and
  that the exile→return happens *within* the ch. III resolution (before the next SBA batch). Pinned by
  `test_saga_return_transformed_chapter_three_no_sacrifice`.
- **`ctx.source` after the immediate exile** — the immediate executor updates `ctx.source` to the exile
  id then to the new battlefield id, so any subsequent effect in the same ability resolves against the
  returned object (CR 400.7j). No roster card has a subsequent clause, but keep the update for
  correctness (mirrors `MoveZone` `:2841`).
- **Wire fingerprints** — re-pin from the failing gates' expected output; never hand-compute. A stale
  `PROTOCOL_SCHEMA_FINGERPRINT` or sentinel hash reds the suite until copied.
```
