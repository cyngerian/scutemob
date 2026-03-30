# Primitive Batch Review: PB-36 -- Evasion/Protection Extensions

**Date**: 2026-03-29
**Reviewer**: primitive-impl-reviewer (Opus)
**CR Rules**: 509.1b (blocking restrictions), 702.16 (protection), 702.111 (menace)
**Engine files reviewed**: `state/types.rs`, `state/hash.rs`, `rules/combat.rs`, `effects/mod.rs`, `cards/card_definition.rs`, `cards/helpers.rs`, `state/mod.rs`, `lib.rs`, `tools/replay-viewer/src/view_model.rs`
**Card defs reviewed**: 16 (bloodghast, carrion_feeder, phoenix_chick, skrelv_defector_mite, vishgraz_the_doomhive, skrevls_hive, white_suns_twilight, signal_pest, gingerbrute, emrakul_the_promised_end, greensleeves_maro_sorcerer, sword_of_body_and_mind, cryptic_coat, untimely_malfunction, teferis_protection, the_one_ring)

## Verdict: needs-fix

PB-36 adds four engine capabilities (CantBlock keyword, CantBeBlockedExceptBy parameterized evasion, BlockingExceptionFilter enum, GrantPlayerProtection effect) and fixes 16 card definitions. The engine changes are solid: combat.rs enforcement is correct for both blocker declaration and Provoke CantBlock impossibility, hash discriminants are correct, view_model.rs display arms are present, and re-exports are complete. The card definitions all match oracle text for the PB-36 scope. Two MEDIUM findings relate to missing Vec length prefixes in hash implementations and a missing Provoke impossibility check for CantBeBlockedExceptBy. Five LOW findings cover test gaps and pre-existing card def TODOs that remain.

## Engine Change Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | **MEDIUM** | `state/hash.rs:5060` | **Missing Vec length prefix in GrantPlayerProtection hash.** The `qualities` Vec is hashed without a length prefix, unlike all other Vec fields in the file. **Fix:** Add `(qualities.len() as u64).hash_into(hasher);` before the for loop. |
| 2 | **MEDIUM** | `state/hash.rs:712` | **Missing Vec length prefix in BlockingExceptionFilter::HasAnyKeyword hash.** The `kws` Vec is hashed without a length prefix. **Fix:** Add `(kws.len() as u64).hash_into(hasher);` before the for loop. |
| 3 | **MEDIUM** | `rules/combat.rs:960-999` | **Missing CantBeBlockedExceptBy check in Provoke requirement-impossibility section.** The Provoke section checks CantBlock, Decayed, CantBeBlocked, Intimidate, Fear, Shadow, Horsemanship, Skulk, but does NOT check CantBeBlockedExceptBy. A provoked creature that doesn't match the exception filter has an impossible requirement that should be skipped. **Fix:** After the CantBeBlocked check (~line 966), add a CantBeBlockedExceptBy check that mirrors the per-blocker validation at lines 712-730: iterate attacker keywords, check if the provoked creature matches the filter, continue if not. |
| 4 | LOW | `simulator/legal_actions.rs:311-322` | **CantBlock creatures not filtered from legal blocker list.** The simulator's blocker eligibility check does not exclude CantBlock/Decayed/Suspected creatures. The engine rejects them at command processing, but the simulator wastes cycles. Pre-existing for Decayed/Suspected; CantBlock extends the gap. **Fix:** Add keyword/designation checks in the blocker eligibility loop (deferred, pre-existing gap). |

## Card Definition Findings

| # | Severity | Card | Description |
|---|----------|------|-------------|
| 5 | LOW | `the_one_ring.rs` | **ETB trigger fires unconditionally instead of "if you cast it".** Oracle: "When The One Ring enters, if you cast it..." -- intervening_if is None. Documented as TODO. Protection fires on reanimate/flicker. Pre-existing gap; PB-36 only added the GrantPlayerProtection effect to the trigger. **Fix:** When Condition::WasCast is added, set `intervening_if: Some(Condition::WasCast)`. |
| 6 | LOW | `the_one_ring.rs:49` | **Draw card count is Fixed(1) instead of burden counter count.** Oracle: "draw a card for each burden counter on The One Ring." The def uses `EffectAmount::Fixed(1)`. Documented as TODO. Pre-existing. **Fix:** When EffectAmount::CountersOnSource variant exists, replace Fixed(1). |
| 7 | LOW | `teferis_protection.rs` | **Only protection-from-everything implemented; 3 of 4 effects missing.** Oracle has: (1) life total can't change, (2) protection from everything, (3) phase out all permanents, (4) exile self. Only (2) is implemented. All documented as TODOs. PB-36 scope was limited to the protection grant. **Fix:** Address in future PBs when prevention effects and mass phase-out are available. |
| 8 | LOW | `untimely_malfunction.rs:49` | **Mode 2 targets DeclaredTarget index 0 which is mode 0's artifact target.** The modal targeting infrastructure uses shared target indices across modes. Mode 2 should target 1-2 creatures but references `EffectFilter::DeclaredTarget { index: 0 }` (mode 0's artifact target). Pre-existing modal targeting gap. **Fix:** When per-mode targeting is implemented, update mode 2's target index. |
| 9 | LOW | `quilled_charger.rs` | **Saddled-attack trigger still a TODO.** Oracle: "Whenever this creature attacks while saddled, it gets +1/+2 and gains menace until end of turn." The trigger condition WhenAttacksWhileSaddled does not exist. Only Saddle 2 keyword is present. Pre-existing; PB-36 did not modify this card. **Fix:** When attack-trigger-while-saddled infrastructure is added, implement the trigger. |

### Finding Details

#### Finding 1: Missing Vec length prefix in GrantPlayerProtection hash

**Severity**: MEDIUM
**File**: `crates/engine/src/state/hash.rs:5060`
**CR Rule**: N/A (architecture invariant: hash correctness for distributed verification)
**Issue**: The `qualities: Vec<ProtectionQuality>` field is hashed by iterating elements without first hashing the length. Every other Vec in hash.rs uses `(vec.len() as u64).hash_into(hasher)` before the element loop (see lines 95, 101, 119, 127, 135, 143, 817, 821, 836, 1005). Without a length prefix, two different-length Vecs could theoretically produce the same hash if their serialized bytes align.
**Fix**: Add `(qualities.len() as u64).hash_into(hasher);` before `for q in qualities {`.

#### Finding 2: Missing Vec length prefix in BlockingExceptionFilter::HasAnyKeyword hash

**Severity**: MEDIUM
**File**: `crates/engine/src/state/hash.rs:712`
**CR Rule**: N/A (architecture invariant: hash correctness)
**Issue**: Same pattern as Finding 1. `HasAnyKeyword(Vec<KeywordAbility>)` hashes without a length prefix.
**Fix**: Add `(kws.len() as u64).hash_into(hasher);` before `for kw in kws {`.

#### Finding 3: Missing CantBeBlockedExceptBy in Provoke requirement-impossibility

**Severity**: MEDIUM
**File**: `crates/engine/src/rules/combat.rs:960-999`
**CR Rule**: 509.1b -- "The defending player checks each creature they control to see whether it's affected by any restrictions (effects that say a creature can't block, or that it can't block unless some condition is met)."
**Issue**: The Provoke requirement-impossibility section (starting ~line 940) determines whether a provoked creature can legally block the provoking attacker. If it can't (due to flying, CantBeBlocked, etc.), the requirement is skipped. The section correctly handles CantBlock (line 952), CantBeBlocked (line 960), Decayed (line 948), Suspected (line 956), Flying (line 941), Intimidate (line 967), Fear (line 984), Shadow (line 994), Horsemanship (line 1000), and Skulk (line 1010). However, it does NOT check `CantBeBlockedExceptBy`. If an attacker has CantBeBlockedExceptBy(Flying/Reach) and provokes a ground creature, the Provoke requirement should be impossible (the ground creature can't block the attacker). Without this check, the requirement might be incorrectly enforced, leading to an illegal game state where a non-matching creature is forced to block.
**Fix**: After the CantBeBlocked check (~line 966), add:
```rust
// CR 509.1b: CantBeBlockedExceptBy -- provoked creature must match filter.
let mut cant_block_due_to_filter = false;
for kw in attacker_chars.keywords.iter() {
    if let KeywordAbility::CantBeBlockedExceptBy(filter) = kw {
        let matches = match filter {
            BlockingExceptionFilter::HasKeyword(req) => provoked_chars.keywords.contains(req.as_ref()),
            BlockingExceptionFilter::HasAnyKeyword(reqs) => reqs.iter().any(|k| provoked_chars.keywords.contains(k)),
        };
        if !matches {
            cant_block_due_to_filter = true;
            break;
        }
    }
}
if cant_block_due_to_filter {
    continue; // Requirement impossible -- skip
}
```

#### Finding 4: CantBlock not filtered from simulator legal blocker list

**Severity**: LOW
**File**: `crates/simulator/src/legal_actions.rs:311-322`
**Issue**: Pre-existing gap now extended. The simulator includes CantBlock creatures in the eligible blocker list, causing the HeuristicBot to attempt illegal blocks that are rejected at command processing.
**Fix**: Deferred (pre-existing for Decayed/Suspected; address holistically).

#### Finding 5: The One Ring ETB fires unconditionally

**Severity**: LOW
**File**: `crates/engine/src/cards/defs/the_one_ring.rs:28`
**Oracle**: "When The One Ring enters, if you cast it, you gain protection from everything until your next turn."
**Issue**: `intervening_if: None` means protection fires on reanimate/flicker/copy, which is incorrect. Documented as TODO in comments.
**Fix**: When `Condition::WasCast` is added, set `intervening_if: Some(Condition::WasCast)`.

## CR Coverage Check

| CR Rule | Implemented? | Tested? | Notes |
|---------|-------------|---------|-------|
| 509.1b (CantBlock) | Yes | Yes | test_cant_block_keyword_prevents_blocking, test_cant_block_keyword_does_not_prevent_attacking |
| 509.1b (CantBeBlockedExceptBy) | Yes | Yes | 4 tests covering HasKeyword, HasAnyKeyword, positive/negative cases, Menace combo |
| 509.1b (Provoke + CantBlock) | Yes | No | Finding 3: CantBeBlockedExceptBy NOT checked in Provoke section |
| 702.16a (FromCardType) | Yes | Yes | test_protection_from_card_type_blocks_instants (Emrakul) |
| 702.16a (FromSubType) | Yes | No | Greensleeves wired but no dedicated subtype protection test |
| 702.16b/e/j (GrantPlayerProtection) | Yes | Yes | test_grant_player_protection_prevents_targeting |
| 702.16j (FromAll player) | Yes | Yes | Teferi's Protection / The One Ring wired |

## Card Def Summary

| Card | Oracle Match | TODOs Remaining | Game State Correct | Notes |
|------|-------------|-----------------|-------------------|-------|
| bloodghast | Yes | 1 (optional return) | Mostly | "you may return" is non-optional (pre-existing) |
| carrion_feeder | Yes | 0 | Yes | Clean |
| phoenix_chick | Yes | 1 (graveyard attack trigger) | Partial | Pre-existing DSL gap for attack-with-3 trigger |
| skrelv_defector_mite | Yes | 1 (activated ability) | Partial | CantBlock correct; activated ability deferred |
| vishgraz_the_doomhive | Yes | 1 (CDA +1/+1 per poison) | Partial | Tokens correct with CantBlock |
| skrevls_hive | Yes | 1 (Corrupted static) | Partial | Tokens correct with CantBlock |
| white_suns_twilight | Yes | 0 | Yes | Clean |
| signal_pest | Yes | 0 | Yes | Clean |
| gingerbrute | Yes | 0 | Yes | Activated evasion correct |
| emrakul_the_promised_end | Yes | 1 (cast trigger) | Partial | Protection from instants correct |
| greensleeves_maro_sorcerer | Yes | 0 | Yes | Dual protection correct |
| sword_of_body_and_mind | Yes | 0 | Yes | Protection + combat trigger correct |
| cryptic_coat | Yes | 0 | Yes | Static +1/+0 and CantBeBlocked correct |
| untimely_malfunction | Yes | 2 (mode 1, target indexing) | Partial | Mode 2 CantBlock correct; modal targeting pre-existing gap |
| teferis_protection | Partial | 3 (life lock, phase out, exile) | Partial | GrantPlayerProtection correct; 3 effects missing |
| the_one_ring | Partial | 3 (intervening if, upkeep loss, draw scaling) | Partial | GrantPlayerProtection correct; 3 gaps remaining |

## Test Coverage Assessment

9 tests in `evasion_protection.rs` covering:
- CantBlock prevents blocking (positive)
- CantBlock does not prevent attacking (negative)
- CantBeBlockedExceptBy HasAnyKeyword allows matching (positive)
- CantBeBlockedExceptBy HasAnyKeyword rejects non-matching (negative)
- CantBeBlockedExceptBy HasKeyword allows matching (positive)
- CantBeBlockedExceptBy HasKeyword rejects non-matching (negative)
- CantBeBlockedExceptBy + Menace stacking (combined restrictions)
- GrantPlayerProtection prevents targeting (positive)
- Protection from card type blocks instants (positive)

Missing tests (LOW):
- CantBlock + Provoke interaction (provoked CantBlock creature requirement impossible)
- GrantPlayerProtection prevents damage (CR 702.16e)
- Protection from subtype (Wizards) -- wired in Greensleeves but no test
