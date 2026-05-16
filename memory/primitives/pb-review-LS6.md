# Primitive Batch Review: PB-LS6 — Loyalty target validation + DestroyAndReanimate + PreventNextUntap

**Date**: 2026-05-15
**Reviewer**: primitive-impl-reviewer (Opus)
**Task**: scutemob-36 (LS-6 of the LOW-sweep campaign)
**CR Rules**: 606.3, 606.4, 606.6, 601.2c (loyalty + target validation); 701.8 (destroy),
400.7 (object identity), 614.1a (replacement), 704.5d (tokens), 603.6a (ETB triggers);
502.2/502.3 (untap step)
**Engine files reviewed**: `rules/engine.rs`, `cards/card_definition.rs`, `effects/mod.rs`,
`rules/turn_actions.rs`, `state/game_object.rs`, `state/hash.rs`
**Card defs reviewed**: `sorin_lord_of_innistrad.rs`, `tamiyo_field_researcher.rs`,
`hands_of_binding.rs` (3)
**Test files reviewed**: `loyalty_target_validation.rs`, `destroy_and_reanimate.rs`,
`prevent_next_untap.rs` (17 tests)

## Verdict: needs-fix

The implementation is functionally correct on all three issues. L01 places target
validation before the loyalty-cost mutation, faithfully mirroring the activated-ability
path. L02's `DestroyAndReanimate` runs the same destruction pipeline as `DestroyPermanent`,
correctly records only graveyard outcomes, and gates phase-2 reanimation on
`!is_token && card_id.is_some()` plus a live graveyard-zone check. L03's per-object
`skip_untap_steps` counter is decremented only on the controller's untap step with the
correct decrement-always-when-positive ordering, and all 15 `GameObject` struct-literal
sites are updated. Hash wiring (HASH 25→26, discriminants 85/86, new field hashed,
history entry) is complete. No HIGH findings. The findings below are all LOW/MEDIUM:
a wrong-CR-citation propagated through the batch, a planned ETB-trigger test that was
not written, and two docstring/coverage mismatches.

## Engine Change Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | LOW | `cards/card_definition.rs:1285`, `effects/mod.rs:1128`, `state/hash.rs:5881` | **Wrong CR citation for "Destroy."** Doc comments cite "CR 701.7" for the destroy keyword action. CR 701.7 is "Create" (tokens); destroy is **CR 701.8**. The sibling `DestroyAll` variant correctly cites 701.8 at `card_definition.rs:1309`. **Fix:** replace "CR 701.7" with "CR 701.8" in the `DestroyAndReanimate` doc comment, the `effects/mod.rs` arm comment, and the `hash.rs` arm comment. |
| 2 | LOW | `effects/mod.rs:1255-1258` | **Catch-all redirect arm silently drops non-graveyard/non-exile redirects.** `DestroyPermanent`'s redirect handler treats any non-Exile/non-Command destination as a death (emits `CreatureDied`/`PermanentDestroyed`). `DestroyAndReanimate`'s `_ =>` arm emits nothing for a redirect to Hand/Library. The behavior is arguably *more* correct (a card bounced to hand did not die), but it is an undocumented divergence from the established `DestroyPermanent` pipeline. **Fix:** add a one-line comment on the `_ =>` arm noting the intentional divergence — a non-graveyard, non-exile redirect produces no death/destroy event because the card did not die and is not reanimated. (No behavior change needed.) |

## Card Definition Findings

| # | Severity | Card | Description |
|---|----------|------|-------------|
| 3 | LOW | `sorin_lord_of_innistrad.rs:56` | **Wrong CR citation.** The -6 comment cites "CR 701.7" for destroy; should be CR 701.8 (see Finding 1). The other citations (601.2c, 400.7) are correct. **Fix:** change "CR 701.7" to "CR 701.8" in the line-56 comment. |

## Test Review Findings

| # | Severity | File | Description |
|---|----------|------|-------------|
| 4 | MEDIUM | `destroy_and_reanimate.rs` | **Planned ETB-trigger test not written.** The plan's L02 test list specified `test_l02_destroy_and_reanimate_runs_etb` — "destroyed creature has an ETB trigger; assert the trigger fires when it is reanimated (PermanentEnteredBattlefield emitted, trigger queued). CR 603.6a." No such test exists. Tests assert the `PermanentEnteredBattlefield` *event* fires but never that a `CardDefinition` ETB *triggered ability* is queued onto the stack — the reanimate path calls `queue_carddef_etb_triggers`, which is untested here. **Fix:** add a test that reanimates a creature whose `CardDefinition` has an ETB triggered ability and asserts the trigger is queued (e.g. a pending trigger or stack object after the effect resolves). |
| 5 | LOW | `destroy_and_reanimate.rs:1-18, 374` | **Docstring/test mismatch.** The file header lists "ETB triggers fire for reanimated permanents (CR 603.6a)" as a covered case (no such test — see Finding 4). Test 6's docstring (`test_l02_multiple_targets_partial_indestructible`) says "one normal planeswalker," but all three objects are built with `card_creature` (creatures); there is no planeswalker in the test. Per `memory/conventions.md` "aspirationally-wrong comments are correctness hazards." **Fix:** remove the unbacked "ETB triggers fire" line from the header (or add Finding 4's test), and correct test 6's docstring to say "one own creature" instead of "one normal planeswalker." |
| 6 | LOW | `prevent_next_untap.rs:1-12` | **Planned integration tests not written; docstring overclaims.** The plan's L03 list specified `test_l03_tamiyo_minus2_freezes_targets` and `test_l03_hands_of_binding_freezes_target` (full card-def integration). Neither exists — all 5 tests exercise `Effect::PreventNextUntap` directly via `execute_effect`, never through the Tamiyo -2 or Hands of Binding card defs. The file header claims "Integration with Hands of Binding card def (tap + freeze rider)." The card defs are simple `Sequence([TapPermanent, PreventNextUntap])` wrappers, so the risk is low, but the docstring is inaccurate. **Fix:** either add the two planned card-def integration tests, or correct the header to drop the "Integration with Hands of Binding card def" claim. |

### Finding Details

#### Finding 1 + 3: Wrong CR citation for the Destroy keyword action

**Severity**: LOW
**Files**: `cards/card_definition.rs:1285`, `effects/mod.rs:1128` (arm comment),
`state/hash.rs:5881`, `cards/defs/sorin_lord_of_innistrad.rs:56`
**CR Rule**: Verified via MCP `get_rule`: CR 701.7 = "Create"; CR 701.8 = "Destroy".
**Issue**: The batch consistently cites "CR 701.7" for the destroy half of
`DestroyAndReanimate`. The correct rule is CR 701.8. The existing `DestroyAll` variant in
the same file (`card_definition.rs:1309`) already cites 701.8 correctly, so this is an
inconsistency within one file.
**Fix**: Replace every "CR 701.7" reference associated with `DestroyAndReanimate`'s
destroy semantics with "CR 701.8". Leave the "+ reanimation" / "CR 400.7" parts intact.
Also occurs in `destroy_and_reanimate.rs` test docstrings (test 1 line 108, test 6 line
373) — those should be corrected to 701.8 as well.

#### Finding 4: Missing ETB-trigger test for reanimated permanents

**Severity**: MEDIUM (per `memory/conventions.md`: a planned-but-absent test that the
file's own header claims is covered is a coverage gap, not cosmetic)
**File**: `crates/engine/tests/destroy_and_reanimate.rs`
**CR Rule**: CR 603.6a — "A permanent's ability that triggers 'when/whenever [it] enters
the battlefield' triggers when that object enters the battlefield."
**Issue**: The reanimate phase (`effects/mod.rs:1370-1376`) calls
`queue_carddef_etb_triggers`, which is the path that makes a reanimated creature's ETB
triggered ability fire. No test exercises this — the closest test only checks the raw
`PermanentEnteredBattlefield` event, which is emitted unconditionally and would pass even
if `queue_carddef_etb_triggers` were never called. This is the exact silent-skip pattern
the conventions warn against: a test named/described as covering ETB triggers that does
not discriminate whether triggers are actually queued.
**Fix**: Add `test_l02_destroy_and_reanimate_runs_etb` — give the destroyed creature a
`CardDefinition` with a `WhenEntersBattlefield` triggered ability, reanimate it via
`DestroyAndReanimate`, and assert the trigger is queued (a `PendingTrigger` or a stack
object after a priority pass). Mirror an existing reanimation ETB-trigger test.

## CR Coverage Check

| CR Rule | Implemented? | Tested? | Notes |
|---------|-------------|---------|-------|
| 606.3 (loyalty timing) | Yes (pre-existing) | Yes | unchanged; test 3/5 exercise it |
| 606.4 (loyalty cost paid) | Yes | Yes | test 2 (loyalty 6→4), test 5 (6→7) |
| 606.6 (insufficient loyalty) | Yes (pre-existing) | Indirect | unchanged; runs before new validation |
| 601.2c (target validation) | Yes — `engine.rs:2303-2322` | Yes | test 1 (illegal land rejected), test 2/3 |
| L01 validate-before-pay ordering | Yes — validation at 2303, mutation at 2345 | Partial | test 4 relies on Rust move semantics, not a post-state assertion (acceptable — see note) |
| 701.8 (destroy) | Yes — `DestroyAndReanimate` phase 1 | Yes | test 1, miscited as 701.7 (Finding 1) |
| 702.12b (indestructible) | Yes — `effects/mod.rs:1136-1147` | Yes | test 5, test 6 |
| 614.1a (replacement redirect) | Yes — `effects/mod.rs:1213-1259` | Yes | test 4 (RIP→exile) |
| 704.5d (tokens don't return) | Yes — `is_token` gate at `effects/mod.rs:1318` | Yes | test 3 |
| 400.7 (new object identity) | Yes — reanimated card is a new ObjectId | Yes | test 1/2 implicitly; L03 test 4 explicitly |
| 603.6a (ETB triggers fire) | Yes — `queue_carddef_etb_triggers` called | **No** | Finding 4 — planned test absent |
| 502.2/502.3 (untap step / skip) | Yes — `turn_actions.rs:1213-1224` | Yes | L03 tests 1,2,3,5 |
| Freeze counter zone-reset (400.7) | Yes — fresh struct literal | Yes | L03 test 4 |
| Freeze stacking | Yes — `saturating_add` | Yes | L03 test 2 |
| Controller's-untap-step only | Yes — loop filters `controller == active` | Yes | L03 test 5 |

Note on test 4 (`test_l01_loyalty_cost_not_paid_on_rejected_activation`): the test cannot
inspect post-rejection state because `process_command` consumes `GameState` by value and
returns it only on `Ok`. The test documents this and relies on (a) the type system
guaranteeing no mutation leaks on `Err`, and (b) the structural ordering — validation at
`engine.rs:2303-2322` strictly precedes the loyalty-cost mutation at `engine.rs:2345`.
This reasoning is sound and independently verified by reading the handler. The test is
weaker than a post-state assertion but not invalid — there is genuinely no observable
post-state on the `Err` path. Acceptable as-is; no finding.

## Card Def Summary

| Card | Oracle Match | TODOs Remaining | Game State Correct | Notes |
|------|-------------|-----------------|-------------------|-------|
| `sorin_lord_of_innistrad.rs` | Yes | 2 (unrelated — emblem on -2, pre-existing) | Yes | -6 uses `DestroyAndReanimate` with 3 explicit `DeclaredTarget` indices; correct because `DeclaredTarget{index}` resolves exactly one target. CR miscite (Finding 3). |
| `tamiyo_field_researcher.rs` | Yes | 2 (unrelated — +1 combat-damage trigger, -7 emblem, pre-existing) | Yes | -2 `Sequence([Tap×2, PreventNextUntap×2])`; correct. CR citation fixed 613.6→502.3 as planned. |
| `hands_of_binding.rs` | Yes | 0 | Yes | Spell effect `Sequence([Tap, PreventNextUntap])`; freeze TODO removed. `TargetCreature` does not enforce "an opponent controls" — correctly documented as a pre-existing out-of-scope gap. |

## Wiring Verification

- `HASH_SCHEMA_VERSION`: 25→26, confirmed at `hash.rs:195`; history entry 26 present at
  `hash.rs:188-194`; sentinel test asserts 26 at `loyalty_target_validation.rs:353`.
- `HashInto for Effect`: discriminant 85 (`DestroyAndReanimate`, hashes target-list length
  + each target + bool) and 86 (`PreventNextUntap`, hashes target) at `hash.rs:5881-5897`.
- `HashInto for GameObject`: `self.skip_untap_steps.hash_into(hasher)` at `hash.rs:1285`.
- `GameObject.skip_untap_steps: u32` with `#[serde(default)]` at `game_object.rs:1174-1175`.
- 15 explicit `GameObject` struct-literal sites updated: `state/mod.rs` ×4 (530, 676, 792,
  1005), `state/builder.rs` ×1 (1099), `rules/resolution.rs` ×6 (4512, 4714, 5430, 6092,
  6305, 6534), `effects/mod.rs` ×4 (3761, 4639, 4807, 6996). Matches the plan's count.
- `DeclaredTarget` resolution: confirmed `resolve_effect_target_list_indexed`
  (`effects/mod.rs:5931`) resolves `DeclaredTarget{index}` to exactly one target — the
  runner correctly chose the `targets: Vec<EffectTarget>` shape and listed 3 indices for
  Sorin -6, resolving the plan's biggest open question correctly.
- `cargo build --workspace` / `cargo test --all` / `clippy` / `fmt`: WIP reports all green
  (2855 tests, +36). Not re-run by reviewer (read-only); the 15 struct-literal sites and
  exhaustive hash arms are the compile-critical surface and all are present.

## Recommended Fix-Phase

1. **Finding 4 (MEDIUM)**: Add `test_l02_destroy_and_reanimate_runs_etb` exercising the
   `queue_carddef_etb_triggers` path. ~30-40 lines.
2. **Findings 1 + 3 (LOW)**: Replace "CR 701.7" → "CR 701.8" in 5 sites (3 engine doc
   comments, Sorin card def, 2 test docstrings).
3. **Finding 5 (LOW)**: Fix `destroy_and_reanimate.rs` header (drop unbacked ETB line or
   add Finding 4's test) and test 6 docstring ("one normal planeswalker" → "one own
   creature").
4. **Finding 6 (LOW)**: Fix `prevent_next_untap.rs` header — drop the "Integration with
   Hands of Binding card def" claim, or add the two planned card-def integration tests.
5. **Finding 2 (LOW)**: Add an explanatory comment on the `_ =>` redirect arm in the
   `DestroyAndReanimate` execution (no behavior change).

None of the findings block correctness. If the coordinator opts to ship as-is, Finding 4
should still be addressed — an untested ETB-trigger path on a reanimation primitive is
the kind of silent gap the PB pipeline exists to catch.
