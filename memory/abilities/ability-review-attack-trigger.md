# Ability Review: Attack Trigger

**Date**: 2026-02-26
**Reviewer**: ability-impl-reviewer (Opus)
**CR**: 508.1m, 508.2a, 508.2b, 508.3a, 508.4, 603.2, 603.3b, 603.5
**Files reviewed**:
- `crates/engine/src/state/game_object.rs:95-127` (TriggerEvent::SelfAttacks)
- `crates/engine/src/cards/card_definition.rs:455-484` (TriggerCondition::WhenAttacks)
- `crates/engine/src/state/hash.rs:883-898` (TriggerEvent hash)
- `crates/engine/src/state/hash.rs:1825-1839` (TriggerCondition hash)
- `crates/engine/src/rules/abilities.rs:290-536` (check_triggers dispatch + collect_triggers_for_event)
- `crates/engine/src/rules/combat.rs:240-306` (handle_declare_attackers: event emit + trigger flush)
- `crates/engine/src/testing/replay_harness.rs:340-451` (enrich_spec_from_def: WhenAttacks block)
- `crates/engine/tests/abilities.rs:1790-2226` (5 tests)
- `crates/engine/tests/combat.rs:740-805` (pre-existing test)

## Verdict: needs-fix

The implementation is functionally correct for the self-referential "whenever this creature
attacks" pattern. The enum variants, hash coverage, dispatch logic, enrichment wiring, trigger
flush timing, and multiplayer behavior are all sound. However, there is one MEDIUM finding
(incorrect CR citation in the dispatch comment) and two LOW findings (CR citation accuracy in
pre-existing test, missing WhenBlocks enrichment parity). No HIGH findings.

## Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | MEDIUM | `abilities.rs:378` | **Wrong CR citation in dispatch comment.** "CR 603.5" is about optional triggers ("may"); the correct citations are CR 508.1m and CR 508.3a. **Fix:** Change comment to `// SelfAttacks: fires on each creature that is declared as an attacker (CR 508.1m, CR 508.3a).` |
| 2 | LOW | `combat.rs:744,748` | **Pre-existing test cites CR 603.5 instead of CR 508.3a.** The test section header and doc comment both say "CR 603.5" but the trigger being tested is "whenever this creature attacks" which is CR 508.3a. CR 603.5 is about optional "may" triggers. |
| 3 | LOW | `replay_harness.rs` | **WhenBlocks has no enrichment block.** WhenAttacks now has an enrichment block (lines 427-448), but the analogous WhenBlocks (card_definition.rs:472) does not. Any card definition using WhenBlocks will silently fail to trigger. Not a bug in this ability but a parity gap to track. |

### Finding Details

#### Finding 1: Wrong CR citation in dispatch comment

**Severity**: MEDIUM
**File**: `crates/engine/src/rules/abilities.rs:378`
**CR Rule**: 603.5 -- "Some triggered abilities' effects are optional (they contain 'may'). These abilities go on the stack when they trigger, regardless of whether their controller intends to exercise the ability's option or not."
**Issue**: The comment at line 378 says `// SelfAttacks: fires on each creature that is attacking (CR 603.5).` CR 603.5 is about optional "may" triggers, which is not what this code does. This code implements CR 508.1m ("Any abilities that trigger on attackers being declared trigger") and CR 508.3a ("An ability that reads 'Whenever [a creature] attacks, ...' triggers if that creature is declared as an attacker"). Incorrect CR citations mislead future developers and violate the project convention that "Tests cite their rules source" (Architecture Invariant 8, extended to code comments per conventions.md).
**Fix**: Change line 378 to: `// SelfAttacks: fires on each creature that is declared as an attacker (CR 508.1m, CR 508.3a).`

#### Finding 2: Pre-existing test cites wrong CR rule

**Severity**: LOW
**File**: `crates/engine/tests/combat.rs:744,748`
**CR Rule**: 508.3a -- "An ability that reads 'Whenever [a creature] attacks, ...' triggers if that creature is declared as an attacker."
**Issue**: The section header at line 744 says "Test 9: SelfAttacks trigger fires when creature attacks (CR 603.5)" and the doc comment at line 748 says "CR 603.5". CR 603.5 is about optional "may" triggers. The correct citation is CR 508.3a (self-referential attack triggers). This is a pre-existing issue, not introduced by this ability implementation.
**Fix**: Update line 744 to `// Test 9: SelfAttacks trigger fires when creature attacks (CR 508.3a)` and line 748 to `/// CR 508.3a -- "Whenever this creature attacks" triggers when it is declared as an attacker and the trigger goes on the stack.`

#### Finding 3: WhenBlocks has no enrichment block (parity gap)

**Severity**: LOW
**File**: `crates/engine/src/testing/replay_harness.rs` (absent code after line 448)
**CR Rule**: 509.1 (declare blockers triggers)
**Issue**: `TriggerCondition::WhenBlocks` exists at card_definition.rs:472, and `TriggerEvent::SelfBlocks` exists at game_object.rs:113, and the dispatch arm exists at abilities.rs:390-401. However, `enrich_spec_from_def` has no block converting `WhenBlocks` to `SelfBlocks`. Any card definition using `WhenBlocks` will have the trigger silently ignored at runtime (the CardDefinition says WhenBlocks but the runtime object never gets a SelfBlocks TriggeredAbilityDef). This is not a bug introduced by this PR but an analogous gap that should be tracked.
**Fix**: Add a WhenBlocks enrichment block after the WhenAttacks block at line 448, following the same pattern. Alternatively, track this as a separate ability gap in the coverage doc.

## CR Coverage Check

| CR Subrule | Implemented? | Tested? | Notes |
|------------|-------------|---------|-------|
| 508.1m (triggers fire on attack declaration) | Yes | Yes | test_attack_trigger_fires_on_declare_attackers, test_attack_trigger_multiple_attackers |
| 508.2a (trigger checked at declaration time only) | Yes (implicit) | No | Engine only fires on AttackersDeclared event, so characteristic changes after declaration cannot retroactively fire. Correct by construction. No explicit test needed. |
| 508.2b (triggers on stack before priority) | Yes | Yes | combat.rs:296-298 flushes triggers before PriorityGiven. Verified in test_attack_trigger_fires_on_declare_attackers (stack has 1 object). |
| 508.3a (self-referential attack trigger) | Yes | Yes | Primary focus of all 5 tests. |
| 508.4 (put onto battlefield attacking = no trigger) | N/A | N/A | Engine does not support "enters attacking" yet. Dispatch correctly fires only on AttackersDeclared events, so this is safe by construction. Documented in plan. |
| 603.2 (automatic trigger on event match) | Yes | Yes | collect_triggers_for_event matches TriggerEvent::SelfAttacks. |
| 603.3b (APNAP ordering) | Yes | Partial | flush_pending_triggers uses APNAP. test_attack_trigger_multiple_attackers verifies 2 triggers from same controller. No cross-controller attack trigger test (because only the active player can declare attackers in standard turn structure). |
| 603.4 (intervening-if) | Yes | No | collect_triggers_for_event checks intervening_if at line 520. No attack trigger test uses intervening_if. Acceptable -- intervening-if is tested generically elsewhere. |
| 603.5 (optional triggers) | Yes (implicit) | No | Optional "may" triggers go on the stack regardless. No attack-trigger test exercises this. Acceptable -- tested generically. |

## Test Coverage Assessment

| Test | Positive/Negative | CR Coverage | Quality |
|------|-------------------|-------------|---------|
| test_attack_trigger_fires_on_declare_attackers | Positive | 508.1m, 508.3a, 603.2 | Good. 4-player multiplayer. Checks AbilityTriggered event, stack count, stack object kind, controller. |
| test_attack_trigger_via_card_definition_enrich_path | Positive | 508.3a (enrichment) | Good. Critical test. Validates enrich_spec_from_def WhenAttacks conversion. Checks trigger count, AbilityTriggered event, stack presence. |
| test_attack_trigger_resolves_draws_card | Positive | 508.3a, 603 (resolution) | Good. End-to-end: declare attacker, all 4 players pass, trigger resolves, controller draws a card. Checks stack empty, AbilityResolved event, hand count. |
| test_attack_trigger_does_not_fire_for_non_attacker | Negative | 508.3a | Good. Creature with SelfAttacks trigger stays behind while a different creature attacks. Verifies no AbilityTriggered for bystander, empty stack. |
| test_attack_trigger_multiple_attackers | Positive | 508.1m, 603.3b | Good. Two attackers with triggers declared simultaneously. Verifies 2 AbilityTriggered events, 2+ stack objects, both source IDs present. |
| (pre-existing) test_603_self_attacks_trigger_fires | Positive | 508.3a | Adequate. 2-player, basic case. Overlaps with new test 1 but in a different test file and with fewer assertions. |

**Missing test scenarios (all LOW priority)**:
- Creature with attack trigger + vigilance (verifies trigger fires even when creature does not tap). Low risk -- dispatch checks SelfAttacks not SelfBecomesTapped.
- Creature with summoning sickness cannot attack (and therefore trigger never fires). Covered by combat validation tests, not attack-trigger-specific.
- Attack trigger with intervening_if clause. Covered by generic intervening-if tests.
