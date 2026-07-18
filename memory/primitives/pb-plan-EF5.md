# Primitive Batch Plan: PB-EF5 — Card-invokable self-transform (`Effect::TransformSelf`)

**Generated**: 2026-07-18
**Primitive**: `Effect::TransformSelf` — a unit `Effect` variant (no target field) that flips the
resolving ability's own source DFC (`ctx.source`) in place, through the existing Transform/DFC
machinery, honoring the CR 701.27f/701.28e once-per-instruction rule.
**CR Rules**: 701.27 (Transform), 701.28 (Convert), 712.4c (meld can't transform), 712.18
(no new object), 702.145b/e (daybound/nightbound), 704.3 (SBA after transform).
**Cards affected**: 11 body-only DFCs in the roster + 1 integrity demote. **Honest clean ship: 2**
(thaumatic_compass flip Complete, docent_of_perfection author Complete) + **1 demote**
(delver_of_secrets, mismarked `Complete`). The other 8 have a **distinct second blocker** that is a
separate out-of-scope primitive; they stay/author `partial` truthfully-marked or are not authored.
**Dependencies**: PB-EF1..EF4 (shipped). Reuses existing `handle_transform`, `Effect::Conditional`,
`Condition::YouControlNOrMoreWithFilter`, `TriggerCondition::AtBeginningOfYourEndStep`,
`WheneverYouCastSpell`.
**Deferred items from prior PBs**: none pointed at PB-EF5. This batch **files** 3 new seeds
(see §7).

> **Coordinator scoping (constraints — not relitigated here):** ship `Effect::TransformSelf` only;
> do NOT add `Effect::TransformNamed` (verified: every roster DFC self-transforms — §5); `CardType::Battle`
> (Invasion of Ikoria) SPLIT OUT → **OOS-EF5-1**; Sephiroth "Super Nova" SPLIT OUT → **OOS-EF5-2**.

---

## 0. TODO sweep (roster-recall gate — MANDATORY, per feedback_planner_roster_recall)

Ran `Grep "TODO.*[Tt]ransform|TransformSelf|self-transform|no Transform variant|only Meld"` over
`crates/card-defs/src/defs/`. **Result: 2 files self-identify the target primitive** — both already
in the roster:
- `thaumatic_compass.rs:53-54,106` — "needs … TransformSelf effect"; completeness note "Effect has no
  Transform variant (only Meld)".
- `delver_of_secrets.rs:7,28` — "requires TopOfLibraryIsType condition + TransformSelf effect".

Broad `Grep -i "transform"` over defs found only **6** files total (thaumatic_compass, delver_of_secrets,
steel_guardian, kenriths_transformation, excise_the_imperfect, beloved_beggar). The last four are
unrelated (Transform is not their gap). **No forced adds beyond the roster.** The other 9 roster DFCs
(bloodline_keeper, docent_of_perfection, edgar_charmed_groom, fable_of_the_mirror_breaker,
grist_voracious_larva, growing_rites_of_itlimoc, legions_landing, nicol_bolas_the_ravager,
westvale_abbey) have **no def file at all** — they are authored fresh (or left unauthored) per §6.

TODO sweep: 2 cards with matching comments, both already in the roster; 0 forced adds.

---

## 1. Primitive Specification

Add `Effect::TransformSelf` — a **unit** variant on the `Effect` enum (mirror `Effect::Meld`,
`card_definition.rs:2061`; no target field). When it executes during a spell/ability resolution it
flips `ctx.source` to its other face **in place** (CR 712.18 — same `ObjectId`, counters/damage/Auras
persist), gated by:
- **CR 701.27c** — non-DFC source → nothing happens.
- **CR 701.27d / 712.10** — back face is instant/sorcery → nothing happens.
- **CR 712.4c** — meld-pair card → nothing happens.
- **CR 702.145b/e** — daybound/nightbound → nothing happens (they only flip via their keyword system).
- **CR 701.27f / 701.28e** — **once-per-instruction**: the source transforms only if it hasn't already
  transformed/converted since the ability was put on the stack. A second `TransformSelf` in the same
  resolving ability is ignored.
- **CR 704.3** — SBAs checked after the flip.

This is the general effect path usable by triggered abilities ("at the beginning of your end step …
transform"), activated abilities ("{cost}: transform"), and conditional effects inside a trigger
("… then if you control 5+ Wizards, transform") — anything that resolves through `execute_effect`.
It complements the existing `Command::Transform` (external transform command), which keeps its own
validation.

### Prior art in the engine (do NOT duplicate)
- `handle_transform` (`engine.rs:1062`) — the Command path; refactor its flip core into a shared helper.
- `StackObjectKind::TransformTrigger { permanent, ability_timestamp }` (`resolution.rs:7149`) — a
  **vestigial** stack kind that already implements the CR 701.27f guard via a stack-placement
  timestamp, but **has no construction site anywhere** (confirmed by grep — dead/prepared code). It is
  prior art for the guard shape; leave it untouched. `Effect::TransformSelf` is the wired-up path.

---

## 2. CR Rule Text (from MCP)

**701.27f** — "If an activated or triggered ability of a permanent that isn't a delayed triggered
ability of that permanent tries to transform it, the permanent does so only if it hasn't transformed
or converted since the ability was put onto the stack. If a delayed triggered ability … tries to
transform that permanent, the permanent does so only if it hasn't transformed or converted since that
delayed triggered ability was created. In both cases, if the permanent has already transformed or
converted, an instruction to do either is ignored."

**701.27c** — non-DFC/non-DFT → nothing happens.
**701.27d** — back face is instant/sorcery → nothing happens.
**701.27g** — "transformed permanent" = a DFC on the battlefield with its back face up.
**701.28a** — convert follows 701.27a–f (same machinery). **701.28e** — same once-per-instruction rule.
**701.28f** — a permanent that "can't transform" also can't convert.

---

## 3. Engine Changes

### Change 1 — add the `Effect::TransformSelf` variant (DSL)
**File**: `crates/card-types/src/cards/card_definition.rs`
**Action**: add a unit variant `TransformSelf` to `enum Effect`, immediately after `Meld`
(`~2061`). Doc-comment it with CR 701.27a/f + 712.18 and the once-per-instruction rule.
Unit variant, no fields (mirror `Meld` exactly).
**Wire consequence**: this reaches the SR-8 fingerprint closure (Characteristics→Effect) and the
GameState hash closure → forces PROTOCOL + HASH bumps (§4).

### Change 2 — extract the shared flip helper
**File**: `crates/engine/src/rules/engine.rs`
**Action**: refactor the **flip core** out of `handle_transform` (`1062`) into a new free helper:

```rust
/// CR 701.27a-g / 712.18: flip a DFC permanent to its other face in place.
/// No new object (CR 712.18). Counters/damage/Auras persist. Runs the CR 704.3
/// SBA check. Returns PermanentTransformed (+ SBA) events, or an empty vec if
/// nothing happens (non-DFC 701.27c, instant/sorcery back 701.27d, meld-pair
/// 712.4c, or daybound/nightbound 702.145). Does NOT validate zone/controller
/// (caller's job) and does NOT run the CR 701.27f once-per-instruction guard
/// (caller's job — see the Effect executor).
pub(crate) fn transform_permanent_in_place(
    state: &mut GameState,
    permanent: crate::state::game_object::ObjectId,
) -> Result<Vec<GameEvent>, GameStateError>
```

Body = the current `handle_transform` tail (lines ~1101–1166): meld-pair guard (712.4c), DFC check
(701.27c), instant/sorcery-back check (701.27d), the flip (`is_transformed = !is_transformed`,
`last_transform_timestamp = timestamp_counter`, `timestamp_counter += 1`), `PermanentTransformed`
event, and the `sba::check_and_apply_sbas` call. **Add a daybound/nightbound no-op guard here**
(return `Ok(vec![])`) so the Effect path respects CR 702.145 — see the Change 3 note on preserving
Command behavior.

`handle_transform` becomes: keep its existing **Command-path validation** (object exists, zone ==
Battlefield, controller == player, and the **daybound/nightbound → `Err`** rejection at `1085–1100`,
so `Command::Transform` behavior is byte-identical) → then `events.extend(transform_permanent_in_place(state, permanent)?)`.
Because the wrapper already `Err`s on daybound before calling the helper, the helper's daybound no-op
never fires on the Command path — no behavior change. **Verify** `mechanics_a_d/daybound.rs` and
`mechanics_m_z/transform.rs` stay green.

### Change 3 — once-per-instruction guard field on `EffectContext`
**File**: `crates/engine/src/effects/mod.rs`
**Action**: add `pub source_transformed_this_resolution: bool` to `struct EffectContext` (`48`).
**Design rationale (cleanest, lowest blast radius):** `EffectContext` is a **transient execution
struct** — it is never serialized into a `Command`/`GameEvent` and never hashed into `GameState`, so a
new field here forces **no** wire/hash bump. `TransformSelf` only ever transforms `ctx.source`, so a
single bool ("has the source flipped during this resolving instruction?") is exactly the CR 701.27f
scope. This is preferred over a resolution-start-timestamp because the bool needs no counter
arithmetic and no `<=`-boundary reasoning (a resolution-start timestamp collides with the value the
first flip writes; avoid it). It propagates correctly through `Effect::Sequence` and
`Effect::Conditional` because both re-use the **same** `&mut ctx` (`mod.rs:3164-3178`) — so
`docent`'s transform-inside-a-Conditional and any "transform … transform" Sequence share the guard.

**Exhaustive struct-literal sites** — the new field must be added to every `EffectContext { … }`
literal or the crate won't compile:
| File | Site | Line | Action |
|------|------|------|--------|
| `effects/mod.rs` | `EffectContext::new` (`Self { … }`) | ~171 | `source_transformed_this_resolution: false` |
| `effects/mod.rs` | `EffectContext::new_with_kicker` | ~207 | `false` |
| `effects/mod.rs` | ForEach EachPlayer/EachOpponent `inner_ctx` | ~3192 | copy from parent: `ctx.source_transformed_this_resolution` |
| `effects/mod.rs` | ForEach default `inner_ctx` | ~3230 | copy from parent |
| `effects/mod.rs` | ctx literal | ~8801 | `false` |

(ForEach copies the parent value so a defensive guard is not lost across a ForEach boundary; no roster
card exercises ForEach+TransformSelf.)

### Change 4 — the `Effect::TransformSelf` executor arm
**File**: `crates/engine/src/effects/mod.rs`
**Action**: add a match arm in `execute_effect_inner` (near the `Effect::Meld` arm; the executor arm
proper is `mod.rs:3910`). Study `Effect::Meld` (uses `ctx.source`) and `StackObjectKind::TransformTrigger`
(`resolution.rs:7149`) for the flip + guard shape.

```rust
// CR 701.27a/f, 712.18: flip the resolving ability's own source DFC in place.
Effect::TransformSelf => {
    // CR 701.27f / 701.28e: once-per-instruction — a second TransformSelf in the
    // same resolving ability is ignored.
    if !ctx.source_transformed_this_resolution {
        if let Ok(evs) =
            crate::rules::engine::transform_permanent_in_place(state, ctx.source)
        {
            // Only latch the guard if a flip actually occurred (a no-op on a
            // non-DFC / meld / daybound source must not consume the instruction).
            if evs.iter().any(|e| matches!(e, GameEvent::PermanentTransformed { .. })) {
                ctx.source_transformed_this_resolution = true;
            }
            events.extend(evs);
        }
    }
}
```

**CR**: 701.27a/f + 712.18. Note: the executor does NOT validate zone/controller — for an on-card
self-transform the source is its own permanent; if it has left the battlefield the helper's DFC/zone
reads make it a no-op (a transformed permanent that leaves is a new object; `ctx.source` LKI on the
old id yields no flip — correct, CR 400.7). SBAs are also re-checked by the resolution caller after
`execute_effect` returns; the in-helper SBA check is idempotent within the batch (harmless).

### Change 5 — hash discriminant for the new Effect variant
**File**: `crates/engine/src/state/hash.rs`
**Action**: add an arm in `impl HashInto for Effect` (the block containing `Effect::Meld => 53u8…` at
`6147`): `Effect::TransformSelf => Nu8.hash_into(hasher)`. **Assign the next unused Effect
discriminant** — scan the block for the current max (it is non-contiguous; the highest observed is
`SetNoMaximumHandSize = 92` at `6551`, so **use 93** unless a higher one exists — verify by scanning
the whole `Effect` arm). A collision or omission is caught by the sentinel-hash + protocol-fingerprint
gates (§4).

### Change 6 — exhaustive-match confirmation (NOT expected to touch display code)
A new `Effect` variant does **not** touch `tools/tui/src/play/panels/stack_view.rs` or
`tools/replay-viewer/src/view_model.rs` — those match `StackObjectKind`/`KeywordAbility`, not `Effect`
(confirmed: no exhaustive `match … Effect` outside `effects/mod.rs` + `hash.rs`; the simulator has none;
`command.rs` references effect names non-exhaustively). **Proof obligation:** `cargo build --workspace`
must pass — it is the only thing that proves no exhaustive `Effect` match was missed (per gotchas-infra).

---

## 4. Wire-bump checklist (machine-forced)

The new `Effect::TransformSelf` variant is in **both** the SR-8 protocol fingerprint closure
(Characteristics→Effect) and the GameState hash closure (Effect is hashed). Both gates will fail until
re-pinned. EffectContext's new field forces **neither** (transient, unserialized/unhashed).

- [ ] `PROTOCOL_VERSION` 9 → **10** (`rules/protocol.rs:113`).
- [ ] Re-pin `PROTOCOL_SCHEMA_FINGERPRINT` (`rules/protocol.rs:130`) from the failing `protocol_schema`
      test's expected digest; append a `- 10:` History row (protocol.rs history ledger).
- [ ] `HASH_SCHEMA_VERSION` 47 → **48** (`state/hash.rs:430`); append a `HASH_SCHEMA_HISTORY` epoch row
      with both fingerprints from the failing sentinel-hash test.
- [ ] Bump any hardcoded sentinel hashes the version bump moves (run the failing tests; copy expected).
- [ ] `cargo build --workspace` (exhaustive-match proof) + `cargo test --all`.

Let the gates force the numbers; do not hand-guess the fingerprints.

---

## 5. `TransformNamed` — verified NOT needed

Every roster DFC self-transforms (its own `ctx.source`). None transforms a *named other* permanent.
The three "return transformed" cards (edgar, fable, nicol_bolas) are a **different mechanism** —
exile then return as a *new object* entering the battlefield transformed — not a named-other in-place
flip. `TransformNamed` is therefore forbidden speculation (coordinator DECISION 1). Confirmed.

---

## 6. Per-card chain-verification table

TransformSelf is **necessary for all 11** but **sufficient for only 2**. Each "stay partial/blocked"
card has a *distinct, out-of-scope* second blocker (verified absent from the DSL enums below). Oracle
text is canonical (the MCP card tool returns type/keywords only, not oracle text; corroborated by the
WIP recon + existing def notes). Runner: re-confirm P/T and exact wording per card before authoring.

Primitives confirmed to EXIST (checked in `card_definition.rs`): `Effect::Conditional` (`1685`),
`Condition::YouControlNOrMoreWithFilter` (`3633`), `TriggerCondition::AtBeginningOfYourEndStep`
(`3171`), `WheneverYouCastSpell{spell_type_filter,…}` (`3179`), `Effect::CreateToken`, `Effect::MillCards`
(`1375`), `Cost::Sacrifice(TargetFilter)` **single** (`1226`).

Primitives confirmed ABSENT (each is a second blocker): a "top card is instant/sorcery" reveal
condition (only `TopCardIsCreatureOfChosenType` exists, `3660`); an "attacked with N+ creatures"
trigger/condition (`WheneverYouAttack` `3368` is a bare unit, no count; no `Condition` for attacker
count); a **multi-count** sacrifice cost (`Cost::Sacrifice` has no count field); a "tap N other
creatures" activation cost (only `Cost::Tap` = tap self); a "creature card in your graveyard"
condition (only `CardTypesInGraveyardAtLeast`/`SpellMastery` exist); a "look at top N, take a matching
card to hand, bottom the rest" effect (only `Scry`/`Surveil` exist — they reorder, they don't
selectively draw).

| card | file | flip mechanism | 2nd blocker (beyond TransformSelf) | 2nd primitive exists? | verdict |
|------|------|----------------|-----------------------------------|:--:|--------|
| **thaumatic_compass** | exists (`partial`) | end-step trigger; if control 7+ lands, transform | none — `AtBeginningOfYourEndStep` + `YouControlNOrMoreWithFilter{7,lands}` | ✅ | **flip → Complete** |
| **docent_of_perfection** | missing | cast instant/sorcery → make 1/1 Wizard, then if 5+ Wizards transform | none — `WheneverYouCastSpell{[Instant,Sorcery]}` + `CreateToken` + `Conditional{YouControlNOrMoreWithFilter{5,Wizards}, TransformSelf}` | ✅ | **author → Complete** |
| **delver_of_secrets** | exists (**mislabeled `Complete`**) | upkeep: look at top card; if instant/sorcery, transform | "top card is instant/sorcery" reveal condition | ❌ | **DEMOTE `Complete`→`partial`** (integrity — never transforms today; see §6a) |
| **growing_rites_of_itlimoc** | missing | ETB look-top-4 take-a-creature; end-step if 4+ creatures transform | ETB "look at top N, take a creature to hand, bottom rest" effect | ❌ | **partial** (transform half ready; ETB inexpressible) |
| **legions_landing** | missing | ETB make lifelink Vampire; attack with 3+ creatures → transform | "attacked with N+ creatures" trigger/condition | ❌ | **partial** (ETB token fine; flip trigger missing) |
| **westvale_abbey** | missing | `{5},{T}`, Sac **five** creatures → transform | multi-count sacrifice cost (`Cost::Sacrifice` is single) | ❌ | **partial** |
| **bloodline_keeper** | missing | `{T}`, Tap **five other** Vampires → transform | "tap N other creatures" activation cost | ❌ | **partial** |
| **grist_voracious_larva** | missing | ETB mill 3; if a creature card in GY, transform (→ planeswalker back) | "creature card in your graveyard" condition (+ creature→planeswalker transform loyalty risk) | ❌ | **partial** |
| **edgar_charmed_groom** | missing | dies → **return** to battlefield **transformed** (delayed next end step) | return/enter-the-battlefield-transformed (new object, not in-place flip) | ❌ (different mechanism) | **do not author** — OOS-EF5-3 |
| **fable_of_the_mirror_breaker** | missing | Saga ch III: exile, **return transformed** | Saga-chapter exile+return-transformed (new object) | ❌ (different mechanism) | **do not author** — OOS-EF5-3 |
| **nicol_bolas_the_ravager** | missing | `{4}{U}{B}{R}`: exile, **return transformed** | exile+return-transformed (new object) | ❌ (different mechanism) | **do not author** — OOS-EF5-3 |

**Honest discounted ship: 2 Complete** (thaumatic_compass, docent_of_perfection) **+ 1 demote**
(delver). This is well below the queue's speculative "~7–9" — most body-only DFCs are double-blocked,
and the "return transformed" trio needs a fundamentally different primitive. This is the expected
yield-calibration correction (`feedback_pb_yield_calibration`), not a scope failure.

**Runner discretion on the 5 "partial" missing files** (growing_rites, legions_landing,
westvale_abbey, bloodline_keeper, grist): authoring a fresh `partial` file that wires `TransformSelf`
for the transform clause and truthfully marks the surviving blocker is acceptable (W6 policy —
truthful marker, no wrong game state, no gated-stub effects) and gives `TransformSelf` real corpus
usage. If a card's non-transform clauses cannot be modeled without a gated stub, do NOT author it —
leave it for the second-blocker PB and record it in the seed (§7). Prefer authoring 0–2 of these; do
not bloat the corpus with 5 partial stubs.

### 6a. Integrity finding — `delver_of_secrets` is a live-wrong `Complete` def
`delver_of_secrets.rs` ships `completeness: Completeness::Complete` (`51`) but models **no** upkeep
transform trigger (only the `Transform` keyword + `back_face`). It therefore **never transforms** in a
real game — a divergent history (invariant #9), the swan_song failure mode. PB-EF5 does **not** make
it Complete (it needs the absent "top card is instant/sorcery" reveal condition). **Demote it to
`partial`** in this batch with the real blocker:
`Completeness::partial("upkeep transform trigger unmodeled — needs a 'top card is instant/sorcery'
reveal condition; TransformSelf (PB-EF5) is necessary but not sufficient")`. Net clean-coverage
effect: −1 (honest). Do this even though it's not a "flip to Complete" — it removes an integrity
violation, exactly like the swan_song precedent.

---

## 7. Seeds to file (§8-style, into `ef-batch-plan-2026-07-17.md`)

- **OOS-EF5-1** (capability) — `CardType::Battle` / Siege subsystem: card type + defense counters
  (310.4b ETB replacement), 310.6 damage removes defense counters, 310.5/508 combat attackability,
  310.8/310.10 protector designation SBA, 310.7 zero-defense→graveyard SBA, 310.11b Siege
  "last defense counter removed → exile + cast transformed." Unblocks Invasion of Ikoria // Zilortha.
  A whole PB. (Coordinator DECISION 2.)
- **OOS-EF5-2** (capability) — Sephiroth "Super Nova" bespoke keyword action (FF-set DFC back face);
  its own engine project, unrelated to body-only-DFC flips. (Coordinator DECISION 3.)
- **OOS-EF5-3** (capability, **new — surfaced by this plan**) — **return-transformed / enter-the-battlefield-transformed**:
  a permanent is exiled (or dies) and returns as a *new object* already on its back face. NOT
  `TransformSelf` (which flips in place). Needed by edgar_charmed_groom (dies→return transformed,
  delayed), fable_of_the_mirror_breaker (Saga ch III exile+return transformed), nicol_bolas_the_ravager
  ({4}{U}{B}{R} exile+return transformed). Fix shape: a `ReturnTransformed`/`enters_transformed` flag on
  the zone-change/return effect + Saga-chapter integration; new wire type → PROTOCOL bump.
- **OOS-EF5-4** (capability, **new — DFC flip-condition primitives**, batchable) — the distinct second
  blockers that leave 5 roster DFCs `partial` after TransformSelf: (a) a "top card of library is
  instant/sorcery (reveal)" `Condition` (delver_of_secrets); (b) an "attacked with N+ creatures"
  trigger/condition (legions_landing); (c) a **multi-count** sacrifice cost `Cost::Sacrifice { filter,
  count }` (westvale_abbey); (d) a "tap N other permanents matching filter" activation cost
  (bloodline_keeper); (e) a "creature card in your graveyard" `Condition` + verify creature→planeswalker
  transform assigns starting loyalty (grist_voracious_larva); (f) a "look at top N, put a matching card
  into hand, bottom the rest" effect (growing_rites_of_itlimoc ETB). Each is small; several could be
  one PB.

---

## 8. Unit Tests

**File**: `crates/engine/tests/mechanics_m_z/pb_ef5_transform_self.rs` (new module; add `mod
pb_ef5_transform_self;` to `crates/engine/tests/mechanics_m_z/main.rs` — SR-9a: a file with no `mod`
line silently does not compile). Pattern: follow `mechanics_m_z/transform.rs` and `meld.rs`.

Engine-primitive tests (each cites CR; each decoy must fail on exactly the field under test):
- `test_transform_self_flips_source` — a resolving triggered/activated ability with
  `Effect::TransformSelf` flips `ctx.source` front→back; `is_transformed` toggles, same `ObjectId`
  (CR 712.18), `PermanentTransformed` emitted. Reverse case: back→front.
- `test_transform_self_does_not_flip_a_second_dfc` (decoy) — two DFCs on the battlefield; the ability's
  source flips, the *other* controlled DFC does **not** (proves TransformSelf targets only `ctx.source`,
  not "a DFC"). CR 701.27a.
- `test_transform_self_once_per_instruction` — an ability whose effect is
  `Sequence[TransformSelf, TransformSelf]` transforms the source **once** (ends on the back face, not
  back-to-front). CR 701.27f/701.28e. Decoy: reverting the `ctx.source_transformed_this_resolution`
  latch makes it flip twice (back to front) — test asserts final face is back.
- `test_transform_self_non_dfc_noop` — TransformSelf on a non-DFC source: nothing happens, no event,
  guard not latched. CR 701.27c.
- `test_transform_self_instant_sorcery_back_noop` — a DFC whose back is an instant/sorcery: nothing
  happens. CR 701.27d/712.10.
- `test_transform_self_daybound_noop` — TransformSelf on a daybound permanent is a no-op (helper guard).
  CR 702.145. (Also confirm `Command::Transform` on the same still returns `Err` — behavior preserved.)
- `test_command_transform_unchanged` — `Command::Transform` still flips a controlled battlefield DFC and
  still `Err`s on a non-controlled / off-battlefield / daybound permanent (guards the refactor).

Card-def integration tests:
- `test_thaumatic_compass_transforms_at_end_step_with_7_lands` — control 7 lands, pass to end step, the
  trigger resolves `TransformSelf`, front→"Spires of Orazca". With ≤6 lands it does not. CR 714/604.2
  (intervening-if) + 701.27.
- `test_docent_of_perfection_transforms_on_fifth_wizard` — cast instant/sorcery, a 1/1 Wizard token is
  created; on the cast that brings controlled Wizards to ≥5, `TransformSelf` fires (the new token counts,
  Sequence order). With <5 it makes the token and does not flip. CR 603 + 701.27f.
- `test_delver_of_secrets_marked_partial` — assert `delver_of_secrets` def `completeness` is not
  `Complete` (integrity regression guard for §6a).

---

## 9. Verification Checklist

- [ ] `Effect::TransformSelf` variant added (card-types); executor arm + hash arm added (engine).
- [ ] `transform_permanent_in_place` helper extracted; `handle_transform` delegates; `Command::Transform`
      behavior byte-identical (daybound Err preserved).
- [ ] `EffectContext.source_transformed_this_resolution` added to struct + all 5 literal sites.
- [ ] `cargo check` clean.
- [ ] thaumatic_compass flipped Complete; docent_of_perfection authored Complete; delver demoted partial.
- [ ] 0–2 of the second-blocked missing DFCs authored `partial` (optional, runner discretion); rest recorded in OOS-EF5-3/4.
- [ ] PROTOCOL 9→10 + `PROTOCOL_SCHEMA_FINGERPRINT` re-pinned + history row; HASH 47→48 + epoch row; sentinel hashes updated.
- [ ] OOS-EF5-1/2/3/4 filed in `ef-batch-plan-2026-07-17.md`.
- [ ] `cargo build --workspace` (exhaustive-match proof), `cargo test --all`, `cargo clippy --all-targets -- -D warnings`, `cargo fmt --check` + `tools/check-defs-fmt.sh`.
- [ ] No remaining `TransformSelf`/transform TODO in thaumatic_compass; delver note rewritten to real blocker.

---

## 10. Risks & Edge Cases

- **Double-SBA on the effect path** — the helper runs `check_and_apply_sbas`, and the resolution caller
  runs SBAs again after `execute_effect`. Idempotent within a batch; harmless. Do not remove the
  helper's SBA (coordinator wants it centralized).
- **Guard-latch on no-op** — latch `source_transformed_this_resolution` **only** when a
  `PermanentTransformed` event was actually produced, so a no-op (non-DFC/meld/daybound) doesn't
  consume a later legitimate instruction. Encoded in the Change 4 arm.
- **ForEach boundary** — the two `inner_ctx` literals copy the parent flag; no roster card uses
  ForEach+TransformSelf, but the copy keeps the guard defensively correct and the literals must list the
  field to compile.
- **`Command::Transform` regression** — the refactor must keep `handle_transform`'s zone/controller/daybound
  `Err` paths. Pin with `test_command_transform_unchanged`; re-run `mechanics_a_d/daybound.rs`,
  `mechanics_m_z/transform.rs`.
- **creature→planeswalker transform (grist)** — transforming a creature-front into a planeswalker-back
  must assign the back face's starting loyalty as loyalty counters (CR 712 / 306.5b). If the layer
  system does not do this, it is a further blocker for grist — record under OOS-EF5-4(e); do not author
  grist Complete on the assumption it works.
- **`TransformTrigger` dead code** — leave the vestigial `StackObjectKind::TransformTrigger` untouched;
  it is prior art for the guard but has no construction site. Do not wire it (the coordinator chose the
  `Effect` path). Removing it is out of scope.
- **Wire fingerprints** — re-pin from the failing gates' output; never hand-compute. A stale
  `PROTOCOL_SCHEMA_FINGERPRINT` or sentinel hash will red the suite until copied from the expected value.
