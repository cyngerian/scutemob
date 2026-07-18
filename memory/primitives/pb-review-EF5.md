# Primitive Batch Review: PB-EF5 — Card-invokable self-transform (`Effect::TransformSelf`)

**Date**: 2026-07-18
**Reviewer**: primitive-impl-reviewer (Opus)
**Commits reviewed**: `29ba3b3e` (engine), `e3479962` (card defs + tests)
**CR Rules**: 701.27 (Transform), 701.28 (Convert), 712.4c (meld), 712.18 (no new object), 702.145 (day/nightbound), 704.3 (SBA)
**Engine files reviewed**: `rules/engine.rs` (helper extraction), `effects/mod.rs` (executor arm + `EffectContext`), `state/hash.rs` (discriminant 93), `rules/protocol.rs` (PROTOCOL 10), card_definition.rs (variant)
**Card defs reviewed (5)**: `thaumatic_compass.rs`, `docent_of_perfection.rs`, `bloodline_keeper.rs`, `growing_rites_of_itlimoc.rs`, `delver_of_secrets.rs`
**Tests reviewed**: `tests/mechanics_m_z/pb_ef5_transform_self.rs` (12 tests)

## Verdict: needs-fix

The engine primitive is well-built and CR-faithful: the `transform_permanent_in_place`
extraction preserves the `Command::Transform` path byte-for-byte (validation + daybound `Err`
still run before the shared helper, whose day/nightbound no-op therefore never fires on the
Command path), the once-per-instruction bool is fresh per resolution (every resolver builds a
new `EffectContext::new`/`new_with_kicker` with the flag `false`), latches only on a real
`PermanentTransformed` event, and threads correctly through `Sequence`/`Conditional` (shared
`&mut ctx`) and the two ForEach `inner_ctx` copies. Wire bumps (PROTOCOL 9→10, HASH 47→48,
Effect discriminant 93) are machine-forced and correct; discriminant 93 is unique within the
`Effect` HashInto block. Seeds OOS-EF5-1..4 are filed and grist's reclassification to OOS-EF5-3
is justified (real Grist exiles-and-returns-transformed, not an in-place flip). **However, two
`Complete` card defs ship wrong game state** (the project's #1 risk): `docent_of_perfection` has
wrong P/T on both faces, and `thaumatic_compass` was flipped to `Complete` while its back face
(Spires of Orazca) has an inexpressible timing restriction on its tapper. Plus one likely token
subtype error and one documented CR-scope narrowing.

## Engine Change Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| E1 | LOW | `effects/mod.rs:4087-4104` | **Once-per-instruction guard is narrower than CR 701.27f.** The per-resolution bool does not bar a second transform when two abilities of the same permanent are on the stack *simultaneously*. Documented tradeoff, unreachable by roster. **Fix:** none required now; revisit if trigger-copying of transform abilities becomes reachable. |

## Card Definition Findings

| # | Severity | Card | Description |
|---|----------|------|-------------|
| C1 | **HIGH** | `docent_of_perfection.rs:46-47,96-97` | **Wrong P/T on both faces.** Def has front `5/4` and back `6/5`; oracle is Docent of Perfection **2/2** and Final Iteration **5/5**. Ships wrong game state in a `Complete` def. **Fix:** front `power: Some(2), toughness: Some(2)`; back `power: Some(5), toughness: Some(5)` (verify back vs Scryfall). |
| C2 | **HIGH** | `thaumatic_compass.rs:90-103,117` | **`Complete` but Spires of Orazca's tapper omits its timing restriction.** Oracle: "{T}: Tap target creature an opponent controls. Activate this ability only during your turn, before attackers are declared." `TimingRestriction` has only `SorcerySpeed`/`AnyTime` — the restriction is inexpressible, so the engine permits illegal instant-speed tapping on any turn. **Fix:** demote to `partial` naming the inexpressible restriction (like the other double-blocked DFCs), OR add a `TimingRestriction` variant (out of scope → seed). |
| C3 | MEDIUM | `docent_of_perfection.rs:10-14` | **Token subtype likely wrong.** Def mints a "Human Wizard" (name + Human+Wizard subtypes); Docent's token is (to my recollection) a **1/1 blue Wizard** with no Human subtype. Does not affect the flip (Wizard present either way) but mislabels the token and adds spurious Human tribal membership. **Fix:** verify vs Scryfall; if confirmed, drop the Human subtype and rename the token "Wizard". |

## Finding Details

### Finding C1 (HIGH): docent_of_perfection P/T wrong on both faces
**File**: `crates/card-defs/src/defs/docent_of_perfection.rs:46-47` (front), `:96-97` (back)
**Oracle**: Docent of Perfection {3}{U}{U} Insect Horror **2/2**, Flying; Final Iteration Eldrazi Insect **5/5**, Flying.
**Issue**: The def sets front `power: Some(5), toughness: Some(4)` and back `power: Some(6), toughness: Some(5)`. Neither matches the printed card. The task brief itself asserts "Docent of Perfection is 2/2," directly contradicting the def's 5/4 front. CMC is correct ({3}{U}{U}, CMC 5, confirmed from the deck-JSON corpus). Everything else on the card is faithful — the token-then-conditional-transform Sequence (count 3, token counts), the +2/+1-and-flying Wizard anthem on the back, and both cast-triggers are correct (rulings corroborate "three or more Wizards" and "Wizards you control get +2/+1 and have flying"). This is purely a P/T data error, uncaught because the integration tests assert only transform state and face names, never P/T.
**Fix**: front `2/2`, back `5/5` (re-verify the back value against Scryfall — my confidence is high on the front 2/2, moderate on the exact back value).

### Finding C2 (HIGH): thaumatic_compass Complete despite an inexpressible back-face timing restriction
**File**: `crates/card-defs/src/defs/thaumatic_compass.rs:90-103` (Spires tapper), `:117` (`Completeness::Complete`)
**Oracle**: Spires of Orazca — "{T}: Add {C}. {T}: Tap target creature an opponent controls. Activate this ability only during your turn, before attackers are declared."
**CR**: 602.5 (activation timing restrictions). Corroborated by the card's own ruling (2017-09-29) about removing a creature from combat / raid — the tapper is a pre-combat offensive tool.
**Issue**: The tap-creature ability is authored with `timing_restriction: None` (= `AnyTime`). `TimingRestriction` (`card_definition.rs:4122`) exposes only `SorcerySpeed` and `AnyTime`; there is no "only during your turn, before attackers are declared" variant, so the restriction cannot be modeled. The engine will therefore allow the controller to tap an opponent's creature at instant speed on any turn (e.g., tapping a blocker mid-combat or an attacker on the opponent's turn) — behavior the card forbids. This is a `Complete` def producing wrong game state (invariant #9 / W6 policy). The plan (§6) recorded thaumatic_compass's second blocker as "none"; this back-face timing restriction is a genuine second blocker that was missed. This is the same honest-marking situation as the other double-blocked DFCs.
**Fix**: demote `thaumatic_compass` to `Completeness::partial(...)` naming the inexpressible Spires timing restriction, and file it under a flip-condition/timing seed (or extend OOS-EF5-4). Alternatively add a `TimingRestriction` variant for "your turn, before attackers" — but that is a new primitive, out of PB-EF5 scope, and should be its own micro-PB. The transform half (front end-step-style search + `TransformSelf`) is correct and does not need reverting.

### Finding C3 (MEDIUM): docent_of_perfection token subtype
**File**: `crates/card-defs/src/defs/docent_of_perfection.rs:10-30` (`wizard_token`)
**Oracle**: "create a 1/1 blue Wizard creature token."
**Issue**: `wizard_token()` uses `name: "Human Wizard"` and subtypes `{Human, Wizard}`. I believe the printed token is a plain **Wizard** (blue), with no Human subtype. The Wizard subtype is present regardless, so the "three or more Wizards" count and the anthem are unaffected — but the spurious Human subtype and the token name are wrong, and the extra Human membership is exactly the kind of legal-but-wrong tribal detail the campaign guards against.
**Fix**: verify against Scryfall; if the token has no Human subtype, set `name: "Wizard"` and subtypes `{Wizard}` only. (Flagged MEDIUM with an explicit verification caveat — I could not confirm the token subtype from the available tooling; the MCP card tool returns only type/keywords for DFC faces.)

## Engine Change Detail

### E1 (LOW): CR 701.27f scope
CR 701.27f: "the permanent does so only if it hasn't transformed or converted **since the ability was put onto the stack**." The implementation uses a per-resolution `EffectContext.source_transformed_this_resolution` bool. This correctly handles the within-resolution case (a `Sequence`/`Conditional` with two `TransformSelf` flips the source once) and correctly gives each *sequentially resolving* ability a fresh guard. It does **not** capture the cross-resolution scope: if two transform-instruction abilities of the same permanent are placed on the stack simultaneously (e.g., a copied triggered ability), the first resolves and flips, and the second — which per 701.27f should be ignored because the permanent "has already transformed since [the second ability] was put onto the stack" — gets a fresh `false` guard and flips back. `obj.last_transform_timestamp` is already written by the helper and would support the CR-exact fix (compare against a stack-placement/creation timestamp, the shape of the vestigial `StackObjectKind::TransformTrigger`). The plan (§3, Change 3) explicitly chose the bool and the coordinator sanctioned it; no roster card can reach the divergence (it needs external trigger-copying). Logged for traceability only.

## CR Coverage Check

| CR Rule | Implemented? | Tested? | Notes |
|---------|-------------|---------|-------|
| 701.27a/712.18 (flip in place, same ObjectId) | Yes | Yes | `test_transform_self_flips_source` (asserts same id + `PermanentTransformed`) |
| 701.27a (targets ctx.source only) | Yes | Yes | `test_transform_self_does_not_flip_a_second_dfc` (decoy) |
| 701.27f/701.28e (once-per-instruction) | Partial | Yes | intra-resolution only; `test_transform_self_once_per_instruction`. See E1 for cross-resolution gap |
| 701.27c (non-DFC no-op) | Yes | Yes | `test_transform_self_non_dfc_noop` (also asserts guard not latched) |
| 701.27d/712.10 (instant/sorcery back no-op) | Yes | Yes | `test_transform_self_instant_sorcery_back_noop` |
| 712.4c (meld no-op) | Yes | No | helper checks `meld_pair.is_some()`; no dedicated PB-EF5 test (existing meld coverage) |
| 702.145 (daybound no-op via effect; Command still Err) | Yes | Yes | `test_transform_self_daybound_noop` |
| Command::Transform unchanged by refactor | Yes | Yes | `test_command_transform_unchanged` (success + non-controlled/off-bf Err) |
| 704.3 (SBA after flip) | Yes | Indirect | helper calls `check_and_apply_sbas`; idempotent w/ resolution-caller SBA (documented) |
| Card integration (thaumatic_compass, docent) | Yes | Yes | end-step intervening-if (7 vs 6 lands); cast-trigger threshold (3 vs 2 Wizards) |
| §6a delver integrity demote | Yes | Yes | `test_delver_of_secrets_marked_partial` |

Test non-vacuity: the second-DFC decoy (C decoy) fails if `TransformSelf` targeted "a DFC" rather than `ctx.source`; the once-per-instruction test fails if the latch is removed (runner confirmed by breaking it); non-DFC/daybound tests assert the guard does **not** latch on a no-op. Tests cite CR. Adequate — but note none assert real-card P/T, which is why C1 went uncaught.

## Card Def Summary

| Card | Oracle Match | Marker | Game State Correct | Notes |
|------|-------------|--------|-------------------|-------|
| `docent_of_perfection` | No (P/T + token subtype) | Complete | **No** | C1 (HIGH P/T both faces), C3 (MEDIUM token subtype); mechanics/anthem/triggers correct |
| `thaumatic_compass` | No (Spires tapper timing) | Complete | **No** | C2 (HIGH — inexpressible back-face timing restriction; should be partial) |
| `bloodline_keeper` | Yes | Complete | Yes | {2}{B}{B} confirmed via CMC 4; 3/3//5/5, 2/2 flying token, +2/+2 anthem, `activation_condition` 5+ Vampires — all correct. Runner's OOS-EF5-4 deviation (real oracle uses `activation_condition`, not "tap N others") is verified correct |
| `growing_rites_of_itlimoc` | Partial (honest) | partial | Yes (no wrong state) | Transform half wired; ETB selective look-and-take truthfully blocked (OOS-EF5-4(f)); back mana abilities correct. Leftover TODO is acceptable for a documented partial |
| `delver_of_secrets` | Partial (honest) | partial | Yes | Correctly demoted from mismarked Complete (§6a); note names the real blocker; 1/1//3/2 flying correct |

## Wire / Seeds Check

- PROTOCOL_VERSION = 10 (`protocol.rs:118`); fingerprint re-pinned + history row. OK.
- HASH_SCHEMA_VERSION = 48 (`hash.rs:435`); `Effect::TransformSelf => 93u8` (`hash.rs:6571`), unique in the Effect block (the other 93u8 is `KeywordAbility::Eternalize`, a separate impl — per-enum namespaced, no collision). History epoch row appended. OK.
- OOS-EF5-1/2/3/4 filed in `ef-batch-plan-2026-07-17.md §9`. grist reclassified OOS-EF5-4(e)→OOS-EF5-3 — justified: real Grist "exile Grist, then return it to the battlefield transformed" is a new-object return, not an in-place flip. OK.

## Bottom line

2 HIGH + 1 MEDIUM + 1 LOW. Both HIGHs are card-def wrong-game-state on `Complete` defs (not engine defects): fix docent's P/T (C1), and demote thaumatic_compass to partial or add the timing primitive (C2). C3 (docent token subtype) needs a Scryfall check. The engine primitive itself is sound and ships correctly; E1 is a documented, unreachable CR-scope narrowing logged only for traceability. Net honest clean-Complete yield for PB-EF5 is lower than the plan's "2 Complete + 1 demote" claim: docent needs a P/T fix and thaumatic_compass is double-blocked, leaving bloodline_keeper as the one clean new Complete plus the delver demote.

---

## Coordinator resolution of review findings (2026-07-18, verified vs cards.sqlite — authoritative)

Reviewer oracle claims were spot-checked against `cards.sqlite` `card_faces` (authoritative
per MEMORY.md; the MCP card tool returns only type/keywords for DFC faces, which is why the
reviewer worked from memory and erred). Ground truth:

- **C1 HIGH (docent P/T 5/4//6/5 "wrong, should be 2/2//5/5") — FALSE POSITIVE.** cards.sqlite:
  Docent of Perfection **5/4**, Final Iteration **6/5**. The def is correct. No change.
- **C3 MEDIUM (docent token should be "1/1 blue Wizard", not Human Wizard) — FALSE POSITIVE.**
  cards.sqlite oracle: "create a 1/1 blue **Human Wizard** creature token." Def is correct
  (also confirms Final Iteration's "Wizards you control get +2/+1 and have flying", which the
  def models). No change.
- **C2 HIGH (thaumatic_compass Complete but back-face ability wrong) — CONFIRMED, different
  root cause than stated.** The reviewer described Spires' ability as "{T}: Tap target creature,
  timing-restricted". The real Spires of Orazca ability (cards.sqlite) is
  "**{T}: Untap target attacking creature an opponent controls and remove it from combat**".
  The def had modeled a **fabricated** "{T}: Tap target creature an opponent controls" — a
  legal-but-wrong Complete def (the #1 project risk). **FIXED**: corrected the modeled ability
  to `UntapPermanent` on a `is_attacking + controller:Opponent` target (both primitives exist),
  corrected the oracle_text + comment to reality, and **demoted the def to `partial`** because
  the "remove it from combat" clause has NO effect primitive (only `Effect::Regenerate`
  references combat removal internally). Filed as **OOS-EF5-4(g)** (`Effect::RemoveFromCombat`).
  Pinned by new test `test_thaumatic_compass_marked_partial`.
- **E1 LOW (guard scope narrower than "since put on stack")** — accepted as-is; documented,
  coordinator-sanctioned, unreachable by any roster card. Logged for traceability.

**Net honest yield after review:** **2 new Complete** (docent_of_perfection, bloodline_keeper)
+ **1 integrity demote** (delver_of_secrets Complete→partial). thaumatic_compass is partial
(front TransformSelf complete, back face blocked). Clean-coverage delta ≈ **+1** (2 new
Complete − 1 demote). growing_rites_of_itlimoc authored partial (real corpus usage of
TransformSelf, ETB clause truthfully blocked).
