# Primitive Batch Plan: PB-OS2 — optional-cost sacrifice power (EF-EF1-A)

**Generated**: 2026-07-18
**Primitive**: Populate `EffectContext.sacrificed_creature_lki` / `sacrifice_fired` from the
**optional-cost** sacrifice path (`Effect::MayPayThenEffect` → `try_pay_optional_cost` →
`pay_optional_cost` → `Cost::Sacrifice` → `sacrifice_permanents_for_player`), mirroring the
already-working activated-cost site and the `Effect::SacrificePermanents` executor. This closes
finding **EF-EF1-A** and makes `EffectAmount::PowerOfSacrificedCreature` (and its EF10 twins) read
the correct layer-resolved LKI in "you may sacrifice a creature; if you do, [X] where X = its
power" optional effects.
**CR Rules**: 608.2h, 608.2i (LKI / look-back), 603.10a (leaves-the-battlefield look-back class),
608.2c (sequential "if you do"), 701.21a (sacrifice), 613.1d (layer-resolved characteristics).
**Cards affected**: 1 (1 existing flip `partial`→`Complete`: `disciple_of_freyalise`; 0 new).
**Dependencies**: PB-EF10 (SHIPPED) — provides `SacrificedCreatureLki`, the `EffectContext`
fields (`sacrificed_creature_lki`, `sacrifice_fired`), and the already-widened
`sacrifice_permanents_for_player -> Vec<SacrificedCreatureLki>`. All present in-tree; verified.
**Deferred items from prior PBs**: EF-EF1-A (PB-EF1 follow-up, deferred at PB-EF10 Step 3.4 — this
batch executes it). No other carry-forward.

---

## Wire impact: NONE (explicit)

- **NO new DSL type.** `SacrificedCreatureLki` already exists in
  `crates/card-types/src/state/types.rs`, is already hashed (`impl HashInto`), and is already a
  field on `StackObject` and `EffectContext` (shipped by PB-EF10). No enum variant, no struct
  field, no new `Effect`/`Condition`/`EffectAmount` is added.
- **NO PROTOCOL bump.** The only signature changes are to two **private `fn`s**
  (`pay_optional_cost`, `try_pay_optional_cost`) — not serialized, not in the SR-8 protocol
  closure. `EffectContext` is runtime resolution scratch (neither hashed nor on the wire; confirmed
  by the EF10 record, hash.rs comment block). Leave `PROTOCOL_VERSION` / `PROTOCOL_SCHEMA_FINGERPRINT`
  and `crates/engine/tests/core/protocol_schema.rs` untouched.
- **NO HASH bump.** No hashed type changes shape; `SacrificedCreatureLki`'s HashInto already exists.
  Leave `HASH_SCHEMA_VERSION` / history and `crates/engine/tests/core/hash_schema.rs` untouched.
- **NO version sentinels** in test files change. Do NOT touch the `PROTOCOL_VERSION == N` /
  `HASH_SCHEMA_VERSION == N` sentinels in `pb_ef10_sacrifice_driven_amounts.rs` or elsewhere.
- **If the runner sees any protocol/hash gate fail, STOP and flag it** — that would mean a hashed
  type was touched unexpectedly, which contradicts this analysis. Do not "just re-pin" a digest.

---

## CR Rule Text (from MCP, authoritative)

- **608.2h** — "If an effect requires information from a specific object … the effect uses the
  current information of that object if it's in the public zone it was expected to be in; if it's no
  longer in that zone, or if the effect has moved it from a public zone to a hidden zone, the effect
  uses the object's last known information." → the sacrificed creature's power is read as **LKI**
  (it is in the graveyard by the time `then` computes X).
- **608.2i** — the look-back exception: an effect reading a *previous* game state uses that prior
  state, and the object need not still be in its old zone. Together with 608.2h this is why the LKI
  is **captured before** `move_object_to_zone` (inside `sacrifice_permanents_for_player`, already
  done) with layer effects resolved, and never re-read from the graveyard.
- **603.10a** — names the look-back-in-time trigger/effect class (leaves-the-battlefield etc.); the
  "sacrifice a creature; if you do, X = its power" clause reads the creature's last battlefield
  existence, consistent with 608.2h/i.
- **608.2c** — instructions are followed in written order; "you may sacrifice a creature. If you do,
  …" is two sequential steps, the second gated on the first. On the optional path the gating is the
  `then` arm of `MayPayThenEffect` (it runs only if the cost was paid) — no `Condition` needed.
- **701.21a** — sacrifice moves the permanent from battlefield directly to its owner's graveyard;
  not destruction. (Already implemented in `sacrifice_permanents_for_player`.)
- **613.1d** — layer-resolved characteristics: an anthem/`+X/+0` on the creature counts toward the
  captured power (the capture site uses layer-resolved chars; the decoy test pins this).

---

## Current state (verified in-tree, do not re-discover)

- `crates/engine/src/effects/mod.rs:7368-7372` — `EffectAmount::PowerOfSacrificedCreature =>
  ctx.sacrificed_creature_lki.first().map(|l| l.power).unwrap_or(0)`. Works once ctx is populated.
- `crates/engine/src/effects/mod.rs:3445-3476` — `Effect::SacrificePermanents` executor already
  sets `ctx.sacrifice_fired` and `ctx.sacrificed_creature_lki` (the mirror to copy).
- `crates/engine/src/effects/mod.rs:7832-7841` — `sacrifice_permanents_for_player` already
  **returns** `Vec<SacrificedCreatureLki>` (pre-zone-move, layer-resolved). Its return is already
  consumed on the mandatory path but **discarded** on the optional path.
- `crates/engine/src/effects/mod.rs:8128-8143` — `pay_optional_cost`'s `Cost::Sacrifice` arm
  discards the vec via `let _ = sacrifice_permanents_for_player(...)` with the explicit EF-EF1-A
  deferral comment. **This is the bug.**
- **Caller audit (grep-confirmed; widening is contained):**
  - `try_pay_optional_cost` — exactly **one** caller: `effects/mod.rs:3408` (the
    `Effect::MayPayThenEffect` executor).
  - `pay_optional_cost` — three call sites: (a) `8180` from `try_pay_optional_cost`; (b) `8146`
    recursion inside its own `Cost::Sequence` arm; (c) `8077` inside `can_pay_optional_cost`'s
    `Cost::Sequence` scratch-simulation probe (return discarded — this is a payability probe on a
    throwaway `state.clone()`).

---

## Engine Changes (single commit; `crates/engine/src/effects/mod.rs`)

### Change 1 — `pay_optional_cost` returns `Vec<SacrificedCreatureLki>`
**File**: `crates/engine/src/effects/mod.rs:8096-8162`
**Action**: change the return type from `()` to `Vec<crate::state::types::SacrificedCreatureLki>`
and make the `match cost { .. }` the function's tail expression:
- `Cost::Sacrifice(filter)` arm (8128-8143): return the vec from `sacrifice_permanents_for_player`
  (delete the `let _ =`); replace the EF-EF1-A deferral comment with a "PB-OS2 (EF-EF1-A closed):
  the returned LKI is threaded up to the MayPayThenEffect executor" note citing CR 608.2h/608.2i.
- `Cost::Sequence(costs)` arm (8144-8148): accumulate across sub-costs —
  `let mut acc = vec![]; for c in costs { acc.extend(pay_optional_cost(state, pid, c, source, events)); } acc`.
- Every other arm (`Cost::Mana`, `Cost::PayLife`, `Cost::DiscardCard`, and the
  `Cost::Tap | SacrificeSelf | ExileSelf | Forage | RemoveCounter | DiscardSelf | ExileFromHand |
  ExileSelfFromHand | Exert` unreachable arm): return `vec![]`.
**CR**: 608.2h/608.2i — the vec carries the pre-zone-move layer-resolved LKI captured inside
`sacrifice_permanents_for_player`.

### Change 2 — fix `pay_optional_cost`'s two non-`try` callers
**File**: `crates/engine/src/effects/mod.rs`
- **8077** (inside `can_pay_optional_cost` scratch probe): prefix with `let _ =` — this is a
  throwaway payability simulation; discard the returned vec.
- **8146** (the `Cost::Sequence` recursion) is already handled by Change 1 (it is the `acc.extend`
  call). No separate edit.

### Change 3 — `try_pay_optional_cost` returns `Option<Vec<SacrificedCreatureLki>>`
**File**: `crates/engine/src/effects/mod.rs:8168-8182`
**Action**: change return type from `bool` to
`Option<Vec<crate::state::types::SacrificedCreatureLki>>`, preserving the existing
`can_pay → pay → success` semantics:
```rust
if !can_pay_optional_cost(state, pid, cost, source) {
    return None;                       // didn't/can't pay
}
Some(pay_optional_cost(state, pid, cost, source, events))
```
`None` = didn't pay (was `false`); `Some(lki)` = paid (was `true`), `lki` empty for non-sacrifice
costs. Keep the doc comment's "pay when able is a deterministic, replayable choice" note.

### Change 4 — `Effect::MayPayThenEffect` executor records into ctx before `then`
**File**: `crates/engine/src/effects/mod.rs:3404-3413`
**Action**: replace the `if try_pay_optional_cost(..) { ctx.controller = pid; execute_effect_inner(..) }`
body with:
```rust
if let Some(sacrificed) = try_pay_optional_cost(state, pid, cost, Some(ctx.source), events) {
    ctx.controller = pid;                                  // existing payer rebind — keep
    // PB-OS2 (CR 608.2c/608.2h): the optional "if you do" arm reads the LKI of what was
    // just sacrificed. Set before execute_effect_inner so EffectAmount::{Power,Toughness,
    // ManaValue}OfSacrificedCreature inside `then` resolve correctly. Empty for non-sac costs.
    ctx.sacrifice_fired = !sacrificed.is_empty();
    ctx.sacrificed_creature_lki = sacrificed;
    execute_effect_inner(state, then, ctx, events);
}
```
**Keep intact**: the `Some(ctx.source)` source threading (PB-EF1 `exclude_self`), the
`let original_controller = ctx.controller;` capture (3403), the per-payer `ctx.controller = pid`
rebind, and the `ctx.controller = original_controller;` restore after the loop (3413).
**Design note (document in the code comment)**: mirroring the `Effect::SacrificePermanents`
executor, `ctx.sacrificed_creature_lki` uses most-recent-sacrifice-wins semantics and is NOT
restored after the loop — the optional-cost path is per-resolution scratch, consistent with the
mandatory path. On the **decline** branch (`None`) ctx is untouched, so no stale value is written.

### Exhaustive match sites
None. No enum variant, `Effect`, `Condition`, `EffectAmount`, or hashed field is added, so there are
**no** new match arms in `hash.rs`, `view_model.rs`, `stack_view.rs`, `resolve_amount`, or
`check_condition`. The `match cost { .. }` inside `pay_optional_cost` is already exhaustive and
gains no arms (only its arm bodies change to return values). **Still run `cargo build --workspace`**
after the change (SR-3 seal gate) to prove the two signature widenings compile at all call sites.

---

## Card Definition Fix

### disciple_of_freyalise.rs — front face `partial` → `Complete`
**File**: `crates/card-defs/src/defs/disciple_of_freyalise.rs`
**Oracle (front, verified against the in-file header + MCP-consistent text)**: "When this creature
enters, you may sacrifice another creature. If you do, you gain X life and draw X cards, where X is
that creature's power." Back face (Garden of Freyalise) is already Complete — do not touch it.
**Fix**:
1. Replace the empty front-face `abilities: vec![ /* long deferral comment */ ]` with one triggered
   ETB ability (shape follows `springbloom_druid.rs:28-63`, but with a creature `exclude_self` filter
   and a `GainLife`+`DrawCards` `then`):
```rust
AbilityDefinition::Triggered {
    once_per_turn: false,
    trigger_condition: TriggerCondition::WhenEntersBattlefield,
    // CR 118.12 / 109.1: "you may sacrifice another creature. If you do, gain X life and
    // draw X cards, X = its power." PB-OS2 (EF-EF1-A) makes the optional-cost path capture
    // the sacrificed creature's layer-resolved LKI power (CR 608.2h/608.2i).
    effect: Effect::MayPayThenEffect {
        cost: Cost::Sacrifice(TargetFilter {
            has_card_type: Some(CardType::Creature),
            exclude_self: true,               // "another creature" (CR 109.1)
            ..Default::default()
        }),
        payer: PlayerTarget::Controller,
        then: Box::new(Effect::Sequence(vec![
            Effect::GainLife {
                player: PlayerTarget::Controller,
                amount: EffectAmount::PowerOfSacrificedCreature,
            },
            Effect::DrawCards {
                player: PlayerTarget::Controller,
                count: EffectAmount::PowerOfSacrificedCreature,
            },
        ])),
    },
    intervening_if: None,
    targets: vec![],
    modes: None,
    trigger_zone: None,
},
```
   Verify the exact `TargetFilter` field name for creature (`has_card_type: Some(CardType::Creature)`,
   per springbloom's `has_card_type: Some(CardType::Land)`) and that `exclude_self` (card_definition.rs:3126)
   is on the same struct; both confirmed present.
2. Update the top-of-file comment block (lines 8-11) to say the front face IS now implemented (PB-OS2
   closed EF-EF1-A) — remove the "surviving blocker" language.
3. Change `completeness: Completeness::partial(...)` (lines 95-106) to `Completeness::Complete` (match
   the project's exact Complete constructor — grep a neighboring Complete DFC).

### Explicitly NOT in scope (record the one-line reason)
- **birthing_ritual.rs** — stays `partial`. Blocked on the top-7 look/conditional-battlefield-place/
  bottom-random **dig** (OOS-EF10-1 / future PB-OS8), not on the optional-cost power capture. Do not
  touch.
- **ziatora_the_incinerator.rs** — stays `partial`. Its optional sacrifice lives inside a *triggered*
  ability with a reflexive "when you do" and `Triggered` has no `may` field; that is a distinct gap,
  not EF-EF1-A. Do not touch. (Note: it self-identifies the optional-path capture in a TODO, but the
  triggered-may wrapper is the actual blocker; leave the finding filed.)

### Pre-existing TODO sweep (roster-recall gate)
Run at implement time and record the result in the commit body:
```
Grep -i "TODO|BLOCKED|EF-EF1-A" + "optional.*sacrific|sacrifice.*power|PowerOfSacrificedCreature"  crates/card-defs/src/defs/
Grep -i "sacrificed_creature|optional-cost path"  crates/card-defs/src/defs/
```
Planner sweep result (2026-07-18): the optional-cost power-capture gap is self-identified by exactly
two defs — **disciple_of_freyalise.rs** (this batch's flip) and **ziatora_the_incinerator.rs**
(blocked on the distinct triggered-`may` gap, above). No other def names EF-EF1-A or the
optional-cost power capture. **Positive assertion: 1 forced flip (disciple), 0 additional cards.**

---

## Unit Tests

**File**: append to the existing `crates/engine/tests/primitives/pb_ef10_sacrifice_driven_amounts.rs`
(already registered in `primitives/main.rs` — avoids the SR-9a mod-registration hazard; already
imports `execute_effect`, `EffectContext`, `SacrificedCreatureLki`, `GameStateBuilder`, and has an
anthem helper). Add a new section header "PB-OS2: optional-cost sacrifice power (EF-EF1-A)". Each
test cites its CR (invariant #8); each decoy must be proven non-vacuous by temporary revert-and-rerun.
Prefer direct-`Effect::MayPayThenEffect` execution through `execute_effect` over JSON scripts, so the
**pay path itself** (not a hand-built ctx) populates the LKI — that is the code under test.

Add a local power-anthem helper (mirror `anthem_toughness_effect` at line 86 with
`LayerModification::ModifyPower(amount)`):
```rust
fn anthem_power_effect(id: u64, amount: i32) -> ContinuousEffect { /* EffectLayer::PtModify,
    EffectFilter::AllCreatures, LayerModification::ModifyPower(amount), Indefinite */ }
```

### test_may_pay_sacrifice_captures_layer_resolved_power (DECOY — layer + wrong-creature pin)
**CR**: 613.1d / 608.2h. **Setup** (`GameStateBuilder`, p1 active, PreCombatMain):
- p1 controls three objects on the battlefield: a **source** "Disciple" (ObjectId is `ctx.source`; it
  is a creature but `exclude_self` removes it), a **"Fodder"** creature base **2/2** with the *lower*
  ObjectId of the two eligible creatures, and a **"Decoy"** creature base **5/5** with a *higher*
  ObjectId (a wrong-creature bystander that is NOT sacrificed).
- Apply `anthem_power_effect(1, +2)` (AllCreatures, `LayerModification::ModifyPower(+2)`), so Fodder's
  layer-resolved power = 4 and Decoy's = 7. Give p1 ≥4 cards in library.
- Build `ctx = EffectContext::new(p1, <disciple ObjectId>, vec![])`; run through `execute_effect`:
```
Effect::MayPayThenEffect {
    cost: Cost::Sacrifice(TargetFilter { has_card_type: Some(Creature), exclude_self: true, ..default }),
    payer: PlayerTarget::Controller,
    then: Sequence[ GainLife{Controller, PowerOfSacrificedCreature},
                    DrawCards{Controller, PowerOfSacrificedCreature} ],
}
```
  (Deterministic sacrifice picks the lowest eligible ObjectId → Fodder; document this in the test.)
- **Assertions**: life gained == **4** (Fodder's layer-resolved power, not base 2, not Decoy 7/5) AND
  exactly **4** cards moved from library to hand; Fodder is in p1's graveyard; Decoy is still on the
  battlefield. A result of 2 fails the layer pin; 7 or 5 fails the wrong-creature pin.

### test_may_pay_sacrifice_declined_no_capture_no_leak (DECLINE path)
**CR**: 608.2c / 118.12. **Setup**: p1 controls ONLY the "Disciple" source creature (no *other*
creature), so the `exclude_self` creature filter has zero eligible targets →
`can_pay_optional_cost` false → `try_pay_optional_cost` returns `None` → `then` never runs. Give p1 a
known life total and ≥1 card in library.
- Run the **same** `MayPayThenEffect` effect as above, wrapped so a sibling reads the field:
  `Sequence[ MayPayThenEffect{..}, GainLife{Controller, PowerOfSacrificedCreature} ]`.
- **Assertions**: life total unchanged by the `then` (no +X), hand size unchanged (0 draws), the
  sibling `GainLife` gains **0** (proves `ctx.sacrificed_creature_lki` was not populated / no stale
  leak), and `ctx.sacrifice_fired == false` after execution. This test is also the decoy for
  "executor sets the LKI unconditionally" — it fails if the ctx is written on the decline branch.

### test_disciple_of_freyalise_front_face_gains_and_draws_power (card-def integration — recommended)
**CR**: 608.2h. **Setup**: build a `CardRegistry` containing the real `disciple_of_freyalise` def and
a vanilla fodder creature (e.g. 3/3); put a fodder 3/3 on p1's battlefield and ≥3 library cards; move
Disciple onto the battlefield so its ETB triggers; resolve the trigger stack (`drain_stack`), letting
the deterministic optional cost auto-pay (sacrifice the fodder). **Assertions**: p1 gained 3 life and
drew 3 cards; the fodder is in the graveyard; Disciple remains on the battlefield. Follow the
card-integration harness pattern already used in `pb_ef10_sacrifice_driven_amounts.rs` /
`pbp_power_of_sacrificed_creature.rs`. (Load-bearing pair is the two direct-executor tests above; this
one proves the def wiring end-to-end.)

**No version-sentinel test.** Do NOT add or bump any `PROTOCOL_VERSION`/`HASH_SCHEMA_VERSION`
assertion — this batch changes neither.

---

## Close-out actions (runner)

- Replace the EF-EF1-A deferral comment in `pay_optional_cost` (Change 1) with the "closed" note.
- Mark **EF-EF1-A CLOSED** in the source finding docs: check/append a CLOSED banner in
  `memory/card-authoring/w-empty-engine-findings-2026-07-17.md` (~line 16 references EF-EF1-A),
  `memory/primitives/pb-plan-EF10.md` (Step 3.4 optional bonus), and any `w-pb2`/ef-batch note that
  cites EF-EF1-A.
- Update the **PB-OS2** entry in `memory/primitives/oos-retriage-plan-2026-07-18.md` §3 (~line 210)
  to "SHIPPED".
- Regenerate `python3 tools/authoring-report.py`; coverage delta = **+1** (disciple flips to Complete).

---

## Verification Checklist

- [ ] `cargo check -p mtg-engine` clean
- [ ] `cargo build --workspace` clean (SR-3 seal gate; proves both widened signatures compile at all callers)
- [ ] TODO sweep run + result recorded in commit body (positive assertion: 1 flip, 0 extra)
- [ ] disciple_of_freyalise front face wired + `completeness` flipped to Complete; back face untouched
- [ ] Two direct-executor tests (decoy + decline) pass, each proven non-vacuous by revert-and-rerun;
      card-integration test passes
- [ ] `cargo test --all` green (incl. `core card_defs_fmt` + `tools/check-defs-fmt.sh`)
- [ ] `cargo clippy --all-targets -- -D warnings` clean; `cargo fmt --check` + defs-fmt clean
- [ ] **PROTOCOL and HASH versions UNCHANGED**; `core/protocol_schema.rs` + `core/hash_schema.rs`
      untouched; no version-sentinel edits anywhere
- [ ] `python3 tools/authoring-report.py` regenerated; coverage delta = +1
- [ ] EF-EF1-A marked CLOSED in finding docs; PB-OS2 marked SHIPPED in the retriage queue

---

## Risks & Edge Cases

- **Signature-widening blast radius.** Grep-confirmed contained: `try_pay_optional_cost` has exactly
  one caller (MayPayThenEffect executor); `pay_optional_cost` has one `try` caller, one self-recursion
  (Sequence), and one scratch-probe caller (must become `let _ =`). The compiler enumerates any missed
  site.
- **Do NOT regress the PB-EF1 `exclude_self` threading.** Keep `Some(ctx.source)` on the
  `try_pay_optional_cost` call. If it is dropped, disciple would be allowed to sacrifice itself
  ("another creature" violated, CR 109.1). No new test is added for this (PB-EF1 covers it) but the
  runner must not touch that argument.
- **Do NOT regress the payer/controller rebind.** Keep `original_controller` capture/restore and the
  per-payer `ctx.controller = pid`. `then`'s `PlayerTarget::Controller` must resolve to the payer.
- **Most-recent-wins / no restore of `sacrificed_creature_lki`.** Consistent with the
  `Effect::SacrificePermanents` executor; the field is per-resolution scratch. The decline branch does
  not write ctx (the leak test pins this). A hypothetical effect with a mandatory sacrifice *followed
  by* a MayPayThenEffect optional sacrifice in one resolution would see the optional overwrite the
  mandatory — acceptable and matches the "if you do refers to THIS instruction" convention; no such
  card exists in the corpus. Document, don't over-engineer.
- **Object identity (CR 400.7) / no post-move re-read.** The LKI is captured inside
  `sacrifice_permanents_for_player` BEFORE the zone move (already correct from PB-EF10); this batch
  only threads that already-captured vec upward. The decoy test's anthem pins that the captured value
  is layer-resolved, not a graveyard re-read (a graveyard object has lost battlefield-gated layers).
- **`vec![]` for non-sacrifice costs.** Mana/PayLife/DiscardCard optional costs return an empty vec;
  the executor sets `sacrifice_fired = false` and an empty `sacrificed_creature_lki` for them — a
  MayPayThenEffect whose `then` reads `PowerOfSacrificedCreature` after a non-sac cost correctly
  resolves 0 (no regression to existing MayPay users like springbloom, which reads no sac amount).
