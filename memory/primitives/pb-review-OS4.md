# Primitive Batch Review: PB-OS4 — return-transformed / enters-transformed as a NEW object (OOS-EF5-3)

**Date**: 2026-07-19
**Reviewer**: primitive-impl-reviewer (Opus)
**CR Rules**: 400.7 (+400.7j), 712.18, 712.8d/e, 603.7/603.7c, 714.4, 704.5m, 704.5i/306.5b (gap)
**Engine files reviewed**:
- `crates/card-types/src/cards/card_definition.rs` (3 new `Effect` variants)
- `crates/card-types/src/state/stubs.rs` (`DelayedTriggerAction::ReturnFromGraveyardToBattlefieldTransformed`)
- `crates/engine/src/effects/mod.rs` (3 executor arms, :4234/:4319/:4336)
- `crates/engine/src/rules/resolution.rs` (delayed dispatch arm, :7564)
- `crates/engine/src/rules/replacement.rs` (`queue_carddef_etb_triggers` :1312, `register_static_continuous_effects` :2037 — reused, NOT modified)
- `crates/engine/src/rules/layers.rs` (:85-135 back-face substitution — reused)
- `crates/engine/src/rules/turn_actions.rs` (:258-314 upkeep trigger scan — reused)
- `crates/engine/src/rules/protocol.rs` / `crates/engine/src/state/hash.rs` (wire bumps)
**Card defs reviewed**: `edgar_charmed_groom.rs` (Complete), `fable_of_the_mirror_breaker.rs` (Partial) — 2 authored; `nicol_bolas_the_ravager` / `grist_voracious_larva` left unauthored (correct)
**Tests reviewed**: `crates/engine/tests/mechanics_m_z/pb_os4_return_transformed.rs` (14 tests, wired via `mod` in `main.rs:27`)

## Verdict: needs-fix

The primitive mechanics themselves are CR-correct and well-tested: new-object identity (CR 400.7),
counters/Auras do not carry, back-face layer-resolved characteristics (CR 712.8d), the non-DFC guard
(CR ruling), delayed-vs-immediate timing (CR 603.7), and the Saga no-sacrifice result (CR 714.4) are
all implemented and pinned with discriminating decoys. **However there is one HIGH engine/card
defect**: the return-transformed path registers/queues abilities from the **FRONT** card face, and
the engine never gathers a transformed permanent's non-keyword back-face abilities at all. This makes
`edgar_charmed_groom`'s `Complete` marker dishonest — the entire back-face Coffin functions are dead
and the front-face Vampire anthem leaks onto the returned Coffin. There are also two wire/hygiene
MEDIUMs (a double PROTOCOL/HASH bump that violates AC 5040; an unused speculative delayed variant)
and two smaller findings.

## Engine Change Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| E1 | **HIGH** | `effects/mod.rs:4296-4310`, `:4371-4385`; `resolution.rs:7594-7609`; `replacement.rs:2057`, `1415`; `turn_actions.rs:277` | **Return-transformed path uses FRONT-face abilities; back-face non-keyword abilities never function.** `register_static_continuous_effects` and `queue_carddef_etb_triggers` iterate `def.abilities` (front) with no `is_transformed`/`back_face` branch, and the whole engine gathers back-face abilities only for *keywords* (`layers.rs:116` is the sole `back_face.abilities` reader). On the return path this (a) registers the departing card's **front** static/continuous effects onto the returned back-face object, and (b) never queues/scans the back face's triggered/activated/static abilities. **Fix:** file a systemic seed (OOS-OS4-2) for face-aware ability gathering; until fixed, no card whose back face has non-keyword abilities may ship `Complete` on this path (see C1). At minimum, do not register FRONT statics onto an object entering `is_transformed`. |
| E2 | MEDIUM | `rules/protocol.rs:171-185`, `state/hash.rs:496-517` | **Double wire bump violates AC 5040.** PROTOCOL went 18→19→20 and HASH 55→56→57 across two commits, with two History/epoch rows each. AC 5040 requires a single bump per PB. v19/v56 exist only on this unmerged branch (never on main, which is at 18/55), so the append-only ledger's monotonicity is not harmed by collapsing. **Fix:** collapse to PROTOCOL 18→19 and HASH 55→56 with one History row / one epoch row each, re-pin the single fingerprint set from the failing gate output, re-pin `FROZEN_HISTORY_PREFIX_DIGEST`, and update sentinels. |
| E3 | MEDIUM | `card_definition.rs:2130`, `stubs.rs:46`, `effects/mod.rs:4319`, `resolution.rs:7564`, `state/hash.rs` | **Unused speculative variant.** `Effect::ReturnSourceToBattlefieldTransformedNextEndStep` + `DelayedTriggerAction::ReturnFromGraveyardToBattlefieldTransformed` are used by **zero** roster cards (Edgar returns immediately; nicol_bolas returns immediately and is out; no other card needs next-end-step return). This is permanent wire surface with no card to prove it and is the direct cause of the second wire bump (E2). W6 discipline bars speculative machinery. **Recommendation: REMOVE** (see explicit recommendation below). |

## Card Definition Findings

| # | Severity | Card | Description |
|---|----------|------|-------------|
| C1 | **HIGH** | `edgar_charmed_groom.rs:141` | **`Complete` marker is dishonest — back face is entirely non-functional + front anthem leaks.** Per E1: (a) Edgar Markov's Coffin's `AtBeginningOfYourUpkeep` trigger (create Vampire token + bloodline counter + conditional transform — the whole point of the back face) never fires, because the upkeep scan (`turn_actions.rs:277`) reads front `def.abilities`; (b) the front-face "Other Vampires you control get +1/+1" `Static` (line 43) is re-registered onto the returned Coffin object via `register_static_continuous_effects`, so an artifact wrongly grants a Vampire anthem — observably wrong whenever the controller has other Vampires. **Fix:** downgrade to `Completeness::partial(...)` naming the back-face-ability-gathering engine gap and both consequences; file OOS-OS4-2. |
| C2 | MEDIUM | `fable_of_the_mirror_breaker.rs:167-168` | **Partial message misrepresents the back-face ability.** The message says the Reflection of Kiki-Jiki activated ability "inherits Kiki-Jiki['s]… known-wrong gap (TargetFilter has no 'nonlegendary' exclusion)," implying it otherwise works. Per E1 it cannot be activated at all (back-face activated abilities are never gathered). Card is correctly `Partial`, but the stated reason is inaccurate (conventions: aspirational/intended-not-actual descriptions are correctness hazards). **Fix:** amend the partial message to state the back-face activated ability is non-functional pending the OOS-OS4-2 face-aware-gathering gap, not merely mis-filtered. |

### Finding Details

#### E1 / C1: Return-transformed path fires the FRONT face

**CR**: 712.8d — "As a double-faced permanent's back face … it has only the characteristics of that
face" (and by extension its abilities); 400.7 — new object entering transformed.
**Oracle (Edgar Markov's Coffin, back)**: "At the beginning of your upkeep, create a 1/1 white and
black Vampire creature token with lifelink and put a bloodline counter on Edgar Markov's Coffin.
Then if there are three or more bloodline counters on it, remove those counters and transform it."

**Chain walked:** card def → `Effect::ReturnSourceToBattlefieldTransformed` executor
(`effects/mod.rs:4336`) → sets `is_transformed = true` → calls `register_static_continuous_effects`
+ `queue_carddef_etb_triggers` with the card's `card_id`. Both helpers resolve `registry.get(cid)`
→ iterate `def.abilities` (the **front** list) with no face branch. Independently,
`turn_actions.rs:277` (upkeep scan) and every activated/triggered dispatch in `abilities.rs`
(400+ `def.abilities` sites) also read the front list. The only place `back_face.abilities` is read
anywhere in the engine is `layers.rs:116`, and it copies **keyword** abilities only into
`chars.keywords`. Therefore a transformed permanent's back-face triggered/activated/static abilities
never function, and its front statics are (on the return path) re-registered onto the back-face
object.

**Observable wrong state for Edgar (a `Complete` card):** after Edgar dies and returns as the Coffin,
(1) no Vampire tokens are ever created and no bloodline counters accrue (the loop is dead), and (2)
the Coffin — an artifact — grants +1/+1 to the controller's other Vampires via the leaked front
anthem. This is precisely the "legal-but-wrong" failure the project flags as the top pre-alpha risk.

**Scope note:** the root cause is a pre-existing systemic engine limitation, not created by PB-OS4.
But PB-OS4 is the batch that (i) newly calls `register_static_continuous_effects` with a front def on
a back-face-entering object, and (ii) ships a `Complete` card that depends on back-face non-keyword
abilities. The engine fix is out of PB-OS4 scope; the correct in-scope resolution is to downgrade
Edgar to `Partial` and file the seed. The plan's §10 risk analysis only considered back-face **ETB**
triggers (correctly finding neither card has one) and missed back-face **ongoing** triggers /
**statics** — which is how this slipped through to a `Complete` flip.

#### E2: Double wire bump — collapse recommended

`main` is at PROTOCOL 18 / HASH 55 (per CLAUDE.md Current State). The branch is at PROTOCOL 20 /
HASH 57 with History rows `- 19:` and `- 20:` (protocol.rs:171/177) and epoch rows `56`/`57`
(hash.rs:496/508), because the runner committed two effects then added a third mid-PB. The
append-only ledger only forbids rewriting *published* history; 19/56 were never merged. Collapse to a
single 18→19 / 55→56 bump. Confirmed the gate mechanism permits this: the ledger is a `const` array
validated by a prefix digest — recomputing it against a single new row and re-pinning is exactly the
normal bump procedure, not a ledger violation.

#### E3: Unused delayed variant — remove

Tested standalone (tests 6/7) and CR-correct, but no card uses it and none is queued to. Keeping it
is the sole reason a third variant (hence the second bump) exists. See recommendation.

## Explicit Recommendations (requested)

**Double-bump collapse (E2): DO IT.** Collapse PROTOCOL 18→19 and HASH 55→56, single History/epoch
row each, re-pin fingerprints + `FROZEN_HISTORY_PREFIX_DIGEST` + sentinels from the failing gates.
No published history is rewritten (19/56 never left this branch), so no append-only invariant breaks.

**Unused `…NextEndStep` variant (E3): REMOVE IT.** Remove `Effect::ReturnSourceToBattlefieldTransformedNextEndStep`,
`DelayedTriggerAction::ReturnFromGraveyardToBattlefieldTransformed`, the delayed executor arm
(`effects/mod.rs:4319`), the dispatch arm (`resolution.rs:7564`), the 4 hash arms, and tests 6/7.
Rationale: W6 "no speculative machinery," and a real next-end-step-return card can add it in its own
micro-PB with a card to prove it. This also naturally reduces the batch to the two *used* effects
(`ExileSourceAndReturnTransformed`, `ReturnSourceToBattlefieldTransformed`), aligning with a single
wire bump. Net: removing E3 makes E2's collapse trivial.

## CR Coverage Check

| CR Rule | Implemented? | Tested? | Notes |
|---------|-------------|---------|-------|
| 400.7 (new object) | Yes | Yes | test 1 (new id, old id dead), test 7 |
| 400.7 (counters don't carry) | Yes (`move_object_to_zone` resets) | Yes | test 2 + TransformSelf decoy |
| 400.7 / 704.5m (Aura falls off) | Yes | Yes | test 3 |
| 400.7j (find exiled object to return) | Yes | Implicitly | `ctx.source` re-pointed after exile |
| 712.8d (back-face characteristics) | Yes (`layers.rs:97`) | Yes | test 4 + front-name decoy |
| 712.8d (back-face **abilities**) | **No** | Gap | **E1 — only keywords honored** |
| 603.7 (delayed timing) | Yes | Yes | test 6 (unused-by-card variant) |
| 603.7c (delayed = new object) | Yes | Yes | test 7 |
| CR ruling (non-DFC stays) | Yes (all 3 arms guard `is_dfc`) | Yes | test 5 + DFC decoy |
| 714.4 (Saga no-sacrifice) | Yes (no code change needed) | Yes | test 9 + plain-effect decoy |
| 704.5i/306.5b (PW loyalty) | Deferred (OOS-OS4-1) | Guard | test 12 pins nicol/grist not-Complete |

## Card Def Summary

| Card | Oracle Match | TODOs Remaining | Game State Correct | Notes |
|------|-------------|-----------------|-------------------|-------|
| edgar_charmed_groom | Yes (front + back text accurate) | 0 | **No** | Marked `Complete` but back-face Coffin is inert + front anthem leaks (C1/E1). Must be `Partial`. |
| fable_of_the_mirror_breaker | Yes | 2 (ch. I token-trigger, ch. II bounded discard — genuinely inexpressible) | Ch. III correct; back-face activated ability inert | Correctly `Partial`; message inaccurate re back face (C2). |
| nicol_bolas_the_ravager | N/A (unauthored) | — | — | Correctly left out (planeswalker-back loyalty gap, OOS-OS4-1). |
| grist_voracious_larva | N/A (unauthored) | — | — | Correctly left out (loyalty gap + entered-from-graveyard trigger condition). |

## Test Review

Strong primitive coverage with discriminating decoys (counters↔TransformSelf, DFC↔non-DFC,
immediate↔delayed, return-transformed-Saga↔plain-Saga). No `.unwrap()` in library code (executors use
`fizzle_object`/`expect_object`/let-else). SR-9a `mod` line present. **Gap (MEDIUM):** no test drives
a returned object's **back-face** ability. The Edgar integration test (`test_edgar_returns_transformed_immediately`)
stops at "returned as the Coffin, new id, is_transformed" and never advances an upkeep, so the dead
Coffin loop and the leaked anthem are invisible — this is why E1/C1 shipped. **Fix:** add a test that
returns Edgar and advances to the controller's next upkeep asserting a Vampire token is created (will
fail today, exposing the gap) and asserting the Coffin does NOT grant a Vampire anthem; then keep it
green once the card is downgraded/limitation documented, or gate it on OOS-OS4-2.

## Follow-up seeds to file

- **OOS-OS4-1** (already planned): `CardFace.starting_loyalty` + CR 306.5b loyalty assignment on both
  enters-transformed paths + planeswalker back-face authoring → unblocks nicol_bolas / grist.
- **OOS-OS4-2** (NEW, from E1): face-aware ability gathering — a transformed permanent must fire its
  **back face's** triggered/activated/static (non-keyword) abilities, and must NOT fire the front
  face's. Touches `queue_carddef_etb_triggers`, `register_static_continuous_effects`, the
  `turn_actions.rs` upkeep scan, and the `abilities.rs` activated/triggered dispatch sites. Unblocks
  Edgar → Complete and Fable's back-face ability.

## Fix-list (ordered)

1. **E1/C1 (HIGH):** downgrade `edgar_charmed_groom` to `Partial` naming the back-face-gathering gap +
   file OOS-OS4-2. (Do not attempt the systemic engine fix in this PB.)
2. **E3 (MEDIUM):** remove the unused `…NextEndStep` variant + `DelayedTriggerAction` variant +
   dispatch/executor/hash arms + tests 6/7.
3. **E2 (MEDIUM):** collapse the double bump to a single PROTOCOL 18→19 / HASH 55→56 (trivial after
   E3), re-pin fingerprints + prefix digests + sentinels.
4. **C2 (MEDIUM):** correct Fable's partial message.
5. **Test gap (MEDIUM):** add the back-face-upkeep discriminating test (or gate on OOS-OS4-2).
</content>
</invoke>
