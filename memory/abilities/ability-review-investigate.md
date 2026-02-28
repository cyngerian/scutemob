# Ability Review: Investigate

**Date**: 2026-02-28
**Reviewer**: ability-impl-reviewer (Opus)
**CR**: 701.16 (with 111.10f for Clue token definition)
**Files reviewed**:
- `crates/engine/src/cards/card_definition.rs` (lines 325-331, 700-712, 860-894)
- `crates/engine/src/effects/mod.rs` (lines 484-515)
- `crates/engine/src/rules/events.rs` (lines 705-711)
- `crates/engine/src/state/game_object.rs` (lines 191-194)
- `crates/engine/src/state/hash.rs` (lines 1079-1080, 1994-1999, 2548-2551, 2261-2262)
- `crates/engine/src/rules/abilities.rs` (lines 1447-1464)
- `crates/engine/src/cards/definitions.rs` (lines 2920-2936)
- `crates/engine/src/testing/replay_harness.rs` (lines 906-922)
- `crates/engine/tests/investigate.rs` (full file, 491 lines)

## Verdict: clean

The Investigate implementation is correct against CR 701.16a and CR 111.10f. The core
mechanic ("Investigate" means "Create a Clue token") delegates to the existing, well-tested
`clue_token_spec` and `make_token` infrastructure. All hash discriminants are present and
unique. The trigger wiring for "whenever you investigate" follows the established
Surveil/ControllerSurveils pattern. The Thraben Inspector card definition was correctly
updated to use `Effect::Investigate` instead of inline `Effect::CreateToken`. One MEDIUM
finding exists (trigger wiring is untested), but the wiring code is a direct mechanical
copy of the `Surveilled`/`ControllerSurveils` pattern which is tested, and no cards in the
engine currently use `WheneverYouInvestigate`, so the risk is low. Downgrading to LOW per
the "no current consumer" rationale. No HIGH or MEDIUM findings.

## Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | LOW | `tests/investigate.rs` | **No test for WheneverYouInvestigate trigger wiring.** The trigger path (TriggerCondition -> TriggerEvent -> abilities.rs arm) is untested. **Fix:** Add a test with a card using `WheneverYouInvestigate` that verifies the triggered ability fires when `Effect::Investigate` executes. |
| 2 | LOW | `cards/card_definition.rs:338` | **Pre-existing CR citation conflict.** `TapPermanent` cites "CR 701.16" but that is Investigate. The actual Tap rule is CR 701.26a. Not introduced by this change. **Fix:** Change comment from `CR 701.16` to `CR 701.26a`. |

### Finding Details

#### Finding 1: No test for WheneverYouInvestigate trigger wiring

**Severity**: LOW
**File**: `crates/engine/tests/investigate.rs`
**CR Rule**: 701.16a -- "Investigate" means "Create a Clue token."
**Issue**: The implementation adds three components for "whenever you investigate" triggers:
(1) `TriggerCondition::WheneverYouInvestigate` in `card_definition.rs:712`,
(2) `TriggerEvent::ControllerInvestigates` in `game_object.rs:194`, and
(3) the `GameEvent::Investigated` arm in `abilities.rs:1447-1464`.
None of the 6 tests in `investigate.rs` exercise this trigger path. They test the
`Investigated` event emission (test 3) and the token creation (tests 1-2, 5-6), but no
test creates a card with `WheneverYouInvestigate` and verifies its triggered ability fires
when investigating. The wiring is a mechanical copy of the tested `Surveilled`/
`ControllerSurveils` pattern, and no cards in the engine currently use this trigger
condition, so the risk of a latent bug is low.
**Fix:** When a card using `WheneverYouInvestigate` is added (e.g., Lonis,
Cryptozoologist), include a test that verifies the trigger fires. Alternatively, add a
synthetic card test now to exercise the path.

#### Finding 2: Pre-existing CR citation conflict on TapPermanent

**Severity**: LOW
**File**: `crates/engine/src/cards/card_definition.rs:338`
**CR Rule**: 701.26a -- "To tap a permanent, turn it sideways from an upright position."
**Issue**: The `TapPermanent` variant's doc comment says `/// CR 701.16: Tap a permanent.`
but CR 701.16 is Investigate. The correct rule for Tap is CR 701.26a. This is a
pre-existing error not introduced by the Investigate implementation, but the Investigate
work makes it more visible because both now reference "701.16" for different things.
**Fix:** Change the comment at `card_definition.rs:338` from `CR 701.16` to `CR 701.26a`.
This is a one-line cosmetic fix.

## CR Coverage Check

| CR Subrule | Implemented? | Tested? | Notes |
|------------|-------------|---------|-------|
| 701.16a ("Investigate" = create Clue) | Yes | Yes | test_investigate_creates_clue_token |
| 111.10f (Clue token characteristics) | Yes (via clue_token_spec) | Yes | test_investigate_creates_clue_token + existing clue_tokens.rs |
| 111.10f (Clue: colorless) | Yes | Yes | test_investigate_creates_clue_token line 126-128 |
| 111.10f (Clue: Artifact type) | Yes | Yes | test_investigate_creates_clue_token line 129-135 |
| 111.10f (Clue: Clue subtype) | Yes | Yes | test_investigate_creates_clue_token line 136-142 |
| 111.10f (Clue: {2} + sacrifice, no tap) | Yes | Yes | test_investigate_creates_clue_token lines 148-161 |
| 111.10f (Clue: Draw a card) | Yes | Yes | test_investigate_clue_can_be_activated |
| Ruling 2024-06-07 (sequential creation) | Yes | Yes | test_investigate_twice_creates_two_clues |
| Investigate 0 = no-op | Yes | Yes | test_investigate_zero_does_nothing |
| Multiplayer controller correctness | Yes | Yes | test_investigate_multiplayer_correct_controller |
| GameEvent::Investigated emission | Yes | Yes | test_investigate_emits_investigated_event |
| "Whenever you investigate" trigger wiring | Yes | No | Infrastructure added but no test exercises it |

## Hash Coverage

| Type | Variant | Discriminant | File:Line |
|------|---------|-------------|-----------|
| Effect | Investigate | 36 | hash.rs:2549 |
| GameEvent | Investigated | 82 | hash.rs:1996 |
| TriggerEvent | ControllerInvestigates | 16 | hash.rs:1080 |
| TriggerCondition | WheneverYouInvestigate | 20 | hash.rs:2262 |

All hash discriminants are unique and non-conflicting with existing values.
