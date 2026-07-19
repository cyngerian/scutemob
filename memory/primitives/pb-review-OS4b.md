# Primitive Batch Review: PB-OS4b — Face-Aware Ability Gathering for Transformed Permanents

**Date**: 2026-07-19
**Reviewer**: primitive-impl-reviewer (Opus)
**CR Rules**: 712.8d/712.8e (back-face-up permanent has only that face's characteristics/abilities),
712.8a + 400.7 (off-battlefield/zone-change reads are front-face automatically), 714.4 (transformed
Saga is no longer a Saga), 613/604 (statics function only while their face is up), 701.27c (non-DFC
"transform" is a no-op)
**Engine files reviewed**:
- `crates/card-types/src/cards/card_definition.rs` (`effective_abilities` helper, :180)
- `crates/engine/src/rules/face.rs` (NEW — `apply_face_change`, `deregister_face_statics`)
- `crates/engine/src/testing/replay_harness.rs` (`build_face_ability_vectors` extraction :2073, `enrich_spec_from_def` :3333)
- `crates/engine/src/rules/replacement.rs` (`register_static_continuous_effects` face-aware :2056)
- `crates/engine/src/rules/engine.rs` (:1212 in-place, :1435 craft), `effects/mod.rs` (:4299 exile-return)
- `crates/engine/src/rules/turn_actions.rs` (:284 upkeep sweep FIXED; :439/:500/:700 phase sweeps NOT fixed; :1656 daybound)
- `crates/engine/src/rules/resolution.rs` (:665 disturb-enter inline, :1711 ETB register, :7230/:7276/:7377 flip sites)
- `crates/engine/src/rules/mana.rs` (:665 tap-for-mana sweep), `sba.rs` (:843/:889 Saga SBA), `abilities.rs` (:736 cost-reduction guard; CardDefETB consumers :6026/:6845/:6923/:7051)
- `crates/engine/src/rules/protocol.rs` (:178), `crates/engine/src/state/hash.rs` (:504)
**Card defs reviewed** (6): `docent_of_perfection.rs`, `bloodline_keeper.rs`, `growing_rites_of_itlimoc.rs`,
`thaumatic_compass.rs`, `fable_of_the_mirror_breaker.rs`, plus incidental `beloved_beggar`/`brutal_cathar` (spot-checked)
**Tests reviewed**: `crates/engine/tests/mechanics_m_z/pb_os4b_face_aware_abilities.rs` (19 tests, `mod` wired at `main.rs:28`)

> **Note on method**: this environment provided no Bash/cargo tool, so I could not run `cargo test`/
> `clippy`/`build`. All findings below are from direct source inspection + CR/oracle lookups. The
> runner attests 3513 tests green, wire-neutral; compilation/green-suite claims are taken on that
> attestation, everything else is independently verified against source.

## Verdict: needs-fix

The core primitive is implemented correctly and cleanly. The two-channel model is sound: `apply_face_change`
is a single, atomic deregister→flip→rebuild→register choke point, all in-place battlefield flip sites route
through it, the disturb-enter construction path is (correctly) handled inline rather than mislabeled through
the choke point, `deregister_face_statics` does a precise one-per-static structural match (resolving
`EffectFilter::Source` first), and every Channel-B def-index site the plan enumerated (ETB queue, upkeep sweep,
mana-tap sweep, Saga SBA ×2, the CardDefETB consumers, the cost-reduction guard) is face-aware and correctly
gated on live `is_transformed`. The extraction of `build_face_ability_vectors` preserves the SR-34/SF-6 mana↔activated
disjointness and excludes the equipment attach/detach ObjectSpec entries as required. All six card defs are
oracle-accurate and their completeness markers are honest (docent/bloodline pinned `Complete` by execution;
growing_rites/thaumatic/fable honestly `Partial`). Wire is neutral (PROTOCOL 19 / HASH 56). **The one real
gap is the residual (#6): three sibling turn-based sweeps — first-main, postcombat-main, and end-step — were
left indexing the FRONT `def.abilities` on the producer side while their shared CardDefETB consumer was made
face-aware.** This is the same class of bug the PB fixed for the upkeep sweep, it touches two in-scope roster
DFCs (`thaumatic_compass`, `growing_rites_of_itlimoc`), and it is a latent producer/consumer index mismatch.
One MEDIUM (the residual) + one LOW (non-`Static` continuous-effect deregister asymmetry) keep this at needs-fix.

## Engine Change Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| E1 | **MEDIUM** | `rules/turn_actions.rs:439-456` (first-main), `:500-517` (postcombat-main), `:700-717` (end-step) | **Three turn-based trigger sweeps still index FRONT `def.abilities`; same class as the fixed upkeep sweep, and now producer/consumer-inconsistent.** Each produces a `PendingTriggerKind::CardDefETB` whose `ability_index` is an index into `def.abilities` (front), but the shared consumer (`abilities.rs:6845/6923/7051`, and once-per-turn/targets/effect lookups) was made face-aware and re-derives against `effective_abilities(obj.is_transformed)`. Two consequences per CR 712.8d/e: (a) a BACK face with an `AtBeginningOf{FirstMainPhase,PostcombatMain,YourEndStep}` trigger would never fire (producer iterates the front, never sees it) — the exact defect this PB fixed for upkeep; (b) a TRANSFORMED permanent whose FRONT face has such a trigger has that stale front trigger queued every turn and resolved against the back list. **Fix:** route all three producers through `def.effective_abilities(obj.is_transformed)` byte-for-byte as the upkeep sweep at `turn_actions.rs:284` now does. Add a back-face-end-step decoy test mirroring `test_back_upkeep_trigger_fires_only_when_transformed`. |
| E2 | LOW | `rules/face.rs:124-146` (`deregister_face_statics`) vs `replacement.rs:2077-2200` (register) | **Deregister is `Static`-only; register handles the full continuous-effect family.** `register_static_continuous_effects` registers `Static`, `TriggerDoubling`, `SuppressCreatureETBTriggers`, `StaticRestriction`, and `CdaPowerToughness` from the effective face, but `deregister_face_statics` only removes `AbilityDefinition::Static`. So transforming away from a face that declared any of the other four leaks it (and transforming to a face that declares one registers it — asymmetric). The module doc acknowledges this and states no roster card's front OR back DFC face declares any of these; I confirmed none of the 6 roster DFCs do. **Fix (or accept):** either extend `deregister_face_statics` symmetrically (each such effect lives in its own `state.*` collection — `trigger_doublers`, `etb_suppressors`, `restrictions`, `continuous_effects` for CDA) or leave as-is with the existing doc comment. LOW because unreachable by the current roster; document the constraint so a future DFC back face with a stax/doubler/CDA effect is flagged. |

## Card Definition Findings

None. All six defs are oracle-accurate with honest completeness markers and no lingering placeholder
values. (The `TODO` in `growing_rites_of_itlimoc.rs:55-58` is a genuinely-inexpressible front-face
ETB clause the card is correctly `Partial` for — not a stub of this PB's primitive.)

### Finding Details

#### E1: first-main / postcombat-main / end-step sweeps left front-only (the residual, ruled)

**CR**: 712.8d — "While a double-faced permanent has its front face up, it has only the characteristics
of its front face." 712.8e — back-face-up ⇒ only the back face's characteristics (incl. abilities).
**Chain walked** (end-step, the reachable one):
- Producer `carddef_end_step_triggers` (`turn_actions.rs:700`): `def.abilities.iter().enumerate()` filtered
  to `AtBeginningOfYourEndStep`, pushes `PendingTrigger{ability_index: <front idx>, kind: CardDefETB}`.
- Consumer (`abilities.rs:6845/6923/7051`): `def.effective_abilities(obj.is_transformed).get(trigger.ability_index)`.

For the two in-scope transformed DFCs the indices are:
- `thaumatic_compass` (Spires of Orazca): front end-step trigger at front index **2**; back list (Spires) has
  length **2** ⇒ `effective_abilities(true).get(2) == None` ⇒ trigger fizzles.
- `growing_rites_of_itlimoc` (Itlimoc): front end-step trigger at front index **1**; back index 1 is the
  `AddManaScaled` **Activated** ability ⇒ every consumer arm matches only `AbilityDefinition::Triggered { .. }`,
  so it resolves to no ability ⇒ fizzles.

So there is **no observable wrong game state for the current roster** — the stale front trigger lands on an
out-of-bounds or non-`Triggered` slot on the back list and drops. That is why this is MEDIUM, not HIGH.
It is **not** LOW/non-issue because: (1) it is the identical bug class the PB just fixed for the upkeep sweep,
explicitly in scope per `primitive-wip.md` ("make ability gathering face-aware at *every* site that gathers a
battlefield permanent's effective abilities"); (2) a back-face `AtBeginningOfYourEndStep`/first-main/postcombat
trigger would silently never fire; (3) it is a **latent HIGH** — any DFC whose front phase-trigger index collides
with a `Triggered` back ability at the same index would fire the WRONG trigger (real wrong-state); (4) it produces
spurious per-turn end-step trigger activity for `thaumatic_compass`/`growing_rites` once transformed, both cards
this very PB touches and re-pins. The fix is mechanical and identical to the already-applied upkeep fix.

**Fix:** in `first_main_actions`/`postcombat_main_actions`/`end_step_actions`, replace `def.abilities.iter()`
with `def.effective_abilities(obj.is_transformed).iter()` (the loop already holds `obj`). Add a decoy test:
a synthetic DFC whose back face has an `AtBeginningOfYourEndStep` trigger and whose front has none — assert it
fires only when transformed (mirror of `test_back_upkeep_trigger_fires_only_when_transformed`).

## Focus-Area Verification (as requested)

1. **Zero-diff extraction (Change 2a):** `build_face_ability_vectors` reproduces the mana-lowering,
   non-mana activated-lowering (with `cost_to_activation_cost`, timing/condition/zone/once/modes propagation),
   and every triggered-ability conversion (`triggering_creature_filter`, `targets`, `once_per_turn`, `modes`,
   ForEach-adjacent filters, the spell-filter `TargetFilter` forwarding). The SR-34/SF-6 disjointness is preserved:
   the single `mana_ability_lowering` predicate gates BOTH the mana push and the activated-exclusion, so an ability
   lowered to `mana_abilities` cannot also appear in `activated_abilities`. `enrich_spec_from_def` calls it for the
   FRONT face only (`&def.abilities`) and applies mana/activated first, then the Reconfigure/Outlast/keyword-marker
   expansions inline, then the triggered vector — order-preserving. **The equipment attach/detach ObjectSpec entries
   (Reconfigure/Outlast) were correctly EXCLUDED from the shared helper** and remain inline in `enrich`. Because the
   three runtime vectors are index-disjoint from each other, no cross-vector interleaving affects indices. Verified
   faithful by inspection (could not run the before/after test diff, but the extraction is a wholesale move).
2. **`apply_face_change` correctness:** deregister-old → flip+timestamp → rebuild Channel-A from new face →
   register-new; idempotent early-returns on non-live / non-battlefield / unchanged / non-DFC. All in-place
   battlefield flips route through it: `engine.rs:1212` (in-place), `:1435` (craft), `effects/mod.rs:4299`
   (exile-return, with the duplicate standalone register removed), `turn_actions.rs:1656` (daybound),
   `resolution.rs:7230` (TransformTrigger), `:7276` (craft/meld enter-transformed), `:7377`. **No raw
   `is_transformed` flip remains at a battlefield transform boundary.** The one remaining raw write,
   `resolution.rs:665`, is the disturb-cast ETB *construction* path (object not yet a settled battlefield permanent);
   routing it through `apply_face_change` would early-return on the zone guard, so the runner correctly rebuilt
   Channel-A inline there and deferred Channel-B to the generic `register_static_continuous_effects` at `:1716`
   (which reads live `is_transformed`) — avoiding a double-register. Sites 5–8 are correctly identified, not mislabeled.
3. **`deregister_face_statics` (the C1-leak risk):** resolves `EffectFilter::Source → SingleObject(obj_id)` before
   comparing (apples-to-apples with `register`), matches on `(source, layer, duration, modification, filter)`, and
   removes EXACTLY ONE entry per old-face `Static` via `position()` + `remove`. The `test_front_static_removed_on_transform`
   decoy is non-vacuous: its front face genuinely HAS a self-`Static` +1/+0 that the back lacks, and the test manually
   registers it (GameStateBuilder skips ETB) then asserts power drops 3→2 post-transform. `test_transform_there_and_back`
   pins re-registration (2→3 on transform-back).
4. **Index-stability (Channel B):** producer and consumer both key off `effective_abilities(is_transformed)` for the
   FIXED sites (upkeep, mana, Saga, ETB queue, CardDefETB consumers), all gated on live `is_transformed` at consume
   time; the "is_transformed at consume" contract is documented at each consumer. The `get_self_activated_reduction`
   guard (`abilities.rs:736`) correctly SKIPS the front-keyed cost-reduction lookup when `source.is_transformed`,
   closing the index-collision hazard. (E1 is the sole site where producer/consumer diverge.)
5. **Runner's self-reported deviation (3 resolution-time consumers):** in-scope and correct — they are the actual
   CardDefETB effect/target/once-per-turn lookups at trigger *resolution*, sharing the same
   `effective_abilities(obj.is_transformed)` index contract as the stack-build consumers. Not overreach.
6. **Residual — RULED MEDIUM (E1).** See finding detail above.
7. **Probe verification (AC 5058):** `docent_of_perfection` and `bloodline_keeper` are oracle-accurate and honestly
   `Complete`. Tests pin exactly: docent back token-trigger fires (exactly ONE token) and the front "then transform"
   clause is gone (`is_transformed` stays true), back Wizard anthem applies (+2/+1, flying) only post-transform;
   bloodline back `{T}` token ability activatable at index 0, front `{B}: transform` index 1 gone
   (`InvalidAbilityIndex`), back +2/+2 Vampire anthem applies; both `_stays_complete` tests assert the marker.
   `fable_of_the_mirror_breaker`'s corrected message is accurate and precise: back Reflection activated ability is
   now reachable/activatable, card stays `Partial` for chapter I (token-attached ability, TokenSpec gap) and chapter
   II (bounded optional discard), and it states exactly the Kiki-Jiki `nonlegendary` `TargetFilter` residual
   (`except_not_legendary: false`, no nonlegendary exclusion in the filter) rather than claiming full copy correctness.
8. **Wire-neutrality (AC 5040/5059):** `PROTOCOL_VERSION == 19` (`protocol.rs:178`), `HASH_SCHEMA_VERSION == 56`
   (`hash.rs:504`). No new `Effect`/enum variant, no struct field, no `HashInto` arm — `effective_abilities` is a
   read-only method, `apply_face_change`/`build_face_ability_vectors` are internal functions, and
   `register_static_continuous_effects`'s new `is_transformed: bool` param is an internal signature change that does
   not touch the serialized `Command`/`GameEvent`/`Effect` surface. No `PROTOCOL_SCHEMA_FINGERPRINT` change implied.
9. **SR-9a:** `mod pb_os4b_face_aware_abilities;` present at `crates/engine/tests/mechanics_m_z/main.rs:28`.
10. **Edge cases:** mutate+transform is untouched (no change to `layers.rs:246` merged-component path) and noted as a
    known limitation in the plan; copy-of-transformed is unperturbed (`copy.rs` uses copiable front values, not
    `is_transformed` ability gathering — no code touched); off-battlefield front-face reads are covered by
    `test_offbattlefield_uses_front_abilities` and are automatic via CR 400.7 `is_transformed` reset.

## CR Coverage Check

| CR Rule | Implemented? | Tested? | Notes |
|---------|-------------|---------|-------|
| 712.8d/e (back-face abilities — Channel A) | Yes | Yes | docent/bloodline/growing_rites/thaumatic probes + non-DFC/there-and-back decoys |
| 712.8d/e (back-face statics register/deregister) | Yes | Yes | `test_front_static_removed_on_transform`, `test_*_anthem_applies`, there-and-back |
| 712.8d/e (Channel-B: upkeep/mana/ETB/CardDefETB) | Yes | Yes | `test_back_upkeep_trigger_fires_only_when_transformed`, mana-tap probes |
| 712.8d/e (Channel-B: first-main/postcombat/end-step) | **Partial** | **No** | **E1 — producers still front-only; no back-face-phase decoy** |
| 714.4 (transformed Saga not sacrificed) | Yes | Yes | `test_saga_transform_not_sacrificed` + `test_saga_untransformed_is_sacrificed` baseline |
| 400.7 / 712.8a (off-battlefield = front) | Yes | Yes | `test_offbattlefield_uses_front_abilities` |
| 701.27c (non-DFC transform no-op) | Yes | Yes | `test_non_dfc_transform_is_noop_ability_set` |

## Card Def Summary

| Card | Oracle Match | TODOs Remaining | Game State Correct | Notes |
|------|-------------|-----------------|-------------------|-------|
| docent_of_perfection | Yes | 0 | Yes | `Complete`, pinned by 3 probes + `_stays_complete`; back trigger/anthem live, front re-transform gone |
| bloodline_keeper | Yes | 0 | Yes | `Complete`, pinned; back `{T}` activatable, front `{B}:transform` index gone, back anthem live |
| growing_rites_of_itlimoc | Yes | 1 (front ETB look-and-take, genuinely inexpressible) | Yes (back mana now live) | `Partial`; "two mana abilities fully implemented" claim now TRUE post-fix |
| thaumatic_compass | Yes | 0 | Yes (back `{C}` now live) | `Partial` (combat-removal primitive gap OOS-EF5-4g); message unchanged, accurate |
| fable_of_the_mirror_breaker | Yes | 0 (ch I/II are documented Effect::Nothing) | Ch III + back activated reachable | `Partial`; C2 message corrected & precise re Kiki-Jiki nonlegendary residual |
| beloved_beggar / brutal_cathar | Yes | 0 | Yes | keyword-only / empty back faces; transform is an ability-set no-op — no regression |

## Test Review

19 tests, discriminating and decoy-heavy, each citing CR 712.8d/e (and 714.4 / 400.7 where relevant). Positive
cases (back trigger/activated/mana/static function post-transform), negative cases (non-DFC no-op, off-battlefield
front, front-face-no-trigger baseline), and mechanism decoys (static removal, there-and-back re-add, saga-not-
sacrificed with an untransformed-IS-sacrificed baseline, upkeep-fires-only-transformed). Real card defs are used
for probes (via `all_cards()` + `enrich_spec_from_def`), synthetic DFCs for decoys. No `.unwrap()` in a way that
would mask engine bugs (helpers `panic!` on setup failure by design). **One gap (ties to E1):** there is no test
exercising the first-main / postcombat-main / end-step producer sweeps for a transformed permanent — the upkeep
sweep is covered but its three siblings are not. Add the back-face-end-step decoy alongside the E1 fix.

## Residual Ruling (explicit, per task #6)

**The `turn_actions.rs` first-main (~:439) / postcombat-main (~:500) / end-step (~:700) CardDef ability sweeps
are a REAL gap of the same class as the fixed upkeep sweep — filed as E1 (MEDIUM), fix required.** They gather
battlefield-permanent phase-beginning triggers that a transformed permanent could need (a back-face
`AtBeginningOfYourEndStep`/`FirstMainPhase`/`PostcombatMain` trigger would silently never fire), AND they create a
producer(front)/consumer(back) index mismatch for the two in-scope transformed DFCs (`thaumatic_compass`,
`growing_rites_of_itlimoc`), which both carry a FRONT-face `AtBeginningOfYourEndStep` transform trigger. For the
current roster the mismatch resolves to an out-of-bounds (compass, front idx 2 vs back len 2) or a non-`Triggered`
back slot (growing_rites, back idx 1 is Activated), so it fizzles with no observable wrong state today — which is
why it is MEDIUM, not HIGH — but it is a latent HIGH (index collision on a future DFC = wrong trigger fires) and it
leaves the PB's own stated mandate ("every gathering site") incomplete. Fix = route the three producers through
`effective_abilities(obj.is_transformed)` exactly as the upkeep sweep now does, plus a back-face-end-step decoy test.

## Fix-list (ordered)

1. **E1 (MEDIUM):** make the first-main / postcombat-main / end-step CardDef trigger sweeps in `turn_actions.rs`
   face-aware (`def.effective_abilities(obj.is_transformed)`), matching the upkeep sweep. Add a
   back-face-`AtBeginningOfYourEndStep` decoy test.
2. **E2 (LOW):** either extend `deregister_face_statics` to the full continuous-effect family symmetrically with
   `register_static_continuous_effects`, or leave as-is and keep the existing doc constraint (accept). No roster
   card reaches it today.
