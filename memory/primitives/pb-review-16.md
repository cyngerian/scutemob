# Primitive Batch Review: PB-16 — Meld

**Date**: 2026-03-19
**Reviewer**: primitive-impl-reviewer (Opus)
**CR Rules**: 701.42 (Meld action), 712.4 (Meld cards), 712.8g (Melded characteristics), 712.21 (Zone-change splitting), 712.4c (Cannot transform)
**Engine files reviewed**: `effects/mod.rs` (Effect::Meld dispatch), `state/game_object.rs` (meld_component field), `state/mod.rs` (zone-change splitting), `rules/layers.rs` (melded face resolution), `state/hash.rs` (meld_component hashing), `rules/engine.rs` (transform guard), `cards/card_definition.rs` (MeldPair struct, Effect::Meld variant), `cards/helpers.rs` + `cards/mod.rs` + `lib.rs` (MeldPair export)
**Card defs reviewed**: hanweir_battlements.rs, hanweir_garrison.rs, hanweir_the_writhing_township.rs (3 total)

## Verdict: needs-fix

Three findings: one HIGH (phantom exiled cards after meld), one MEDIUM (mana value wrong for melded permanents), and two MEDIUM (TODOs in both card defs for attack triggers). The engine correctly implements melded face characteristics via the layer system, zone-change splitting, and the transform guard (CR 712.4c). Hash support is present. The negative-case tests (partner missing, different controller) are good. However, the meld execution leaves ghost card objects in exile and the mana value calculation violates CR 712.8g.

## Engine Change Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | **HIGH** | `effects/mod.rs:2368-2370` | **Phantom exiled cards after meld.** Two ghost objects remain in exile after melding. **Fix:** remove or consume the exiled objects. |
| 2 | **MEDIUM** | `rules/layers.rs:162` | **Mana value of melded permanent is 0 instead of sum of front faces.** CR 712.8g violated. **Fix:** compute sum of both front face mana values. |

## Card Definition Findings

| # | Severity | Card | Description |
|---|----------|------|-------------|
| 3 | MEDIUM | `hanweir_garrison.rs` | **Attack trigger TODO remaining.** Oracle says "Whenever this creature attacks, create two 1/1 red Human creature tokens tapped and attacking." **Fix:** implement when "tapped and attacking" token creation is available. |
| 4 | MEDIUM | `hanweir_the_writhing_township.rs` | **Attack trigger TODO remaining on back_face.** Oracle says "Whenever Hanweir attacks, create two 3/2 colorless Eldrazi Horror creature tokens tapped and attacking." **Fix:** implement when "tapped and attacking" token creation is available. |
| 5 | LOW | `hanweir_the_writhing_township.rs:31` | **Oracle text minor mismatch.** Card def says "Whenever Hanweir, the Writhing Township attacks" but actual oracle text says "Whenever Hanweir attacks". **Fix:** update oracle_text string to match. |

### Finding Details

#### Finding 1: Phantom exiled cards after meld

**Severity**: HIGH
**File**: `crates/engine/src/effects/mod.rs:2368-2370`
**CR Rule**: 701.42a -- "To meld the two cards in a meld pair, put them onto the battlefield with their back faces up and combined."
**Issue**: The meld effect first exiles both cards via `state.move_object_to_zone(source_id, exile_zone)` and `state.move_object_to_zone(partner_obj_id, exile_zone)`, which creates two new objects in the exile zone (per CR 400.7, new object identity on zone change). Then it creates a THIRD new object on the battlefield as the melded permanent. The two exiled objects remain in exile as phantom copies of the physical cards. This means: (1) effects counting cards in exile see extra cards; (2) "return from exile" effects can target these phantom cards; (3) the game state represents each physical card in two places simultaneously (exile and battlefield via meld_component). The correct behavior is that the cards move from exile to the battlefield combined -- the exile should be transient or the exiled objects should be removed after the melded permanent is created.
**Fix**: After creating the melded permanent on the battlefield, remove both exiled objects from the exile zone and the objects map. The exiled objects exist only to track the zone transition; once the melded permanent is on the battlefield, they should not persist. Specifically, after line 2447 (`state.objects.insert(melded_id, melded_obj)`), look up the new ObjectIds created by the two `move_object_to_zone` calls (they return `(ObjectId, GameObject)` -- capture these return values) and remove those objects from the exile zone and objects map.

#### Finding 2: Mana value of melded permanent is wrong

**Severity**: MEDIUM
**File**: `crates/engine/src/rules/layers.rs:162`
**CR Rule**: 712.8g -- "its mana value is the sum of the mana values of its front faces."
**Issue**: The layer system sets `chars.mana_cost = melded_face.mana_cost.clone()` which is `None` for Hanweir, the Writhing Township (and all melded permanents -- the combined back face has no mana cost). This means `mana_value()` returns 0 for any melded permanent. Per CR 712.8g, the mana value should be the sum of both front face mana values. For Hanweir, this would be 0 (Battlements, a land) + 3 (Garrison, {2}{R}) = 3. This currently produces 0.
**Fix**: In the meld branch of `calculate_characteristics` (layers.rs), after setting `chars.mana_cost`, also look up the meld_component's CardId from `obj.meld_component`, retrieve that card's front face mana cost from the registry, and compute the sum. Set `chars.mana_cost` to a synthetic ManaCost whose `mana_value()` equals the sum (e.g., use a ManaCost with `generic` equal to the sum), OR add a separate `mana_value_override` field to Characteristics. The former approach is simpler: create a synthetic ManaCost `{ generic: source_mv + component_mv, ..Default::default() }` that represents the combined mana value without implying actual colors.

#### Finding 3: Hanweir Garrison attack trigger TODO

**Severity**: MEDIUM
**File**: `crates/engine/src/cards/defs/hanweir_garrison.rs:19-20`
**Oracle**: "Whenever this creature attacks, create two 1/1 red Human creature tokens that are tapped and attacking."
**Issue**: The attack trigger is left as a TODO comment. The card has no abilities implemented. This means Hanweir Garrison on the battlefield does nothing when it attacks, producing wrong game state.
**Fix**: Implement when "tapped and attacking" token creation primitive is available. This is a known DSL gap (creating tokens already attacking), not a PB-16-specific failure. Track as a prerequisite for full meld pair functionality. The trigger itself (WhenThisAttacks -> CreateToken) pattern exists; only the "tapped and attacking" modifier is missing.

#### Finding 4: Hanweir, the Writhing Township attack trigger TODO

**Severity**: MEDIUM
**File**: `crates/engine/src/cards/defs/hanweir_the_writhing_township.rs:35-36`
**Oracle**: "Whenever Hanweir attacks, create two 3/2 colorless Eldrazi Horror creature tokens that are tapped and attacking."
**Issue**: The attack trigger on the melded face is left as a TODO comment. When the melded permanent attacks, it should create two 3/2 Eldrazi Horror tokens tapped and attacking. Currently it does nothing.
**Fix**: Same DSL gap as Finding 3 -- "tapped and attacking" token creation. Implement when available. Note that even Trample and Haste keywords ARE correctly present, so combat characteristics work; only the triggered ability is missing.

#### Finding 5: Oracle text minor mismatch

**Severity**: LOW
**File**: `crates/engine/src/cards/defs/hanweir_the_writhing_township.rs:31`
**Oracle**: "Whenever Hanweir attacks, create two 3/2 colorless Eldrazi Horror creature tokens that are tapped and attacking."
**Issue**: The card def's back_face.oracle_text says "Whenever Hanweir, the Writhing Township attacks" but the actual oracle text uses the short name "Whenever Hanweir attacks".
**Fix**: Change the oracle_text string to use "Whenever Hanweir attacks" to match the actual printed oracle text.

## CR Coverage Check

| CR Rule | Implemented? | Tested? | Notes |
|---------|-------------|---------|-------|
| 701.42a (Meld action) | Partial | Yes | test_meld_basic_exile_and_enter; phantom exile bug |
| 701.42b (Only meld pairs) | Yes | Yes | test_meld_fails_different_controller |
| 701.42c (Conditions not met) | Yes | Yes | test_meld_fails_partner_not_present |
| 712.4a (Meld pair ability) | Partial | Yes | Exile+create works but phantom objects |
| 712.4c (Cannot transform) | Yes | Yes | test_meld_cards_cannot_transform |
| 712.8g (Melded characteristics) | Partial | Yes | test_meld_characteristics_from_back_face; mana value wrong |
| 712.21 (Zone-change splitting) | Yes | Yes | test_meld_zone_change_splitting |
| 712.21a (Graveyard/library order) | N/A | No | Player ordering not implemented (deterministic) |
| 712.21b (Exile timestamp) | No | No | Relative timestamp for two cards on exile not implemented |
| 712.21c (Effect finds both cards) | No | No | Not tested; complex interaction |
| 712.21d (Replacement effects) | No | No | Not tested; edge case |
| 712.21e (Counting objects/cards) | No | No | Not tested |

## Card Def Summary

| Card | Oracle Match | TODOs Remaining | Game State Correct | Notes |
|------|-------------|-----------------|-------------------|-------|
| hanweir_battlements | Yes | 0 | Partial | Meld works but phantom exile bug |
| hanweir_garrison | Yes | 1 | No | Attack trigger missing |
| hanweir_the_writhing_township | Minor mismatch | 1 | Partial | Attack trigger on back_face missing; oracle text uses short name |

## Notes

- The `MeldPair` struct, `Effect::Meld` variant, hash support, layer system integration, and zone-change splitting are all structurally sound. The architecture is correct.
- Mishra, Claimed by Gix (`mishra_claimed_by_gix.rs`) exists in the codebase but is missing `meld_pair` data and has its entire ability as a TODO. This card was NOT listed in PB-16 scope (only hanweir_battlements was), so this is not a finding. It will need to be addressed in a future card authoring batch (it requires a different meld trigger -- meld during combat attack, not via activated ability).
- The "tapped and attacking" token creation is a broader DSL gap affecting multiple cards (not just meld pairs). Findings 3 and 4 are MEDIUM because the cards produce wrong game state, but the fix requires a primitive not yet built.
