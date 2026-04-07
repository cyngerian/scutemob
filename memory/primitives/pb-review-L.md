# Primitive Batch Review: PB-L -- Reveal/X Effects

**Date**: 2026-04-06
**Reviewer**: primitive-impl-reviewer (Opus)
**CR Rules**: CR 305.6 (basic land types / Domain), CR 118.9 (alternative costs), CR 701.20 (reveal), CR 604.3 (CDA), CR 613.4a (CDA P/T layer)
**Engine files reviewed**: `cards/card_definition.rs`, `effects/mod.rs`, `rules/layers.rs`, `rules/casting.rs`, `state/types.rs`, `state/hash.rs`, `testing/replay_harness.rs`
**Card defs reviewed**: 7 (allied_strategies, territorial_maro, coiling_oracle, bounty_of_skemfar, fierce_guardianship, deadly_rollick, flawless_maneuver)

## Verdict: needs-fix

One HIGH finding (Allied Strategies domain count uses controller's lands instead of target player's lands, contradicting oracle text). Two MEDIUM engine findings (CDA path uses base characteristics, misleading doc comment). One MEDIUM card def finding (Bounty of Skemfar oracle_text field diverges from actual oracle). One MEDIUM test gap (no layer-resolved domain test).

## Engine Change Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | **HIGH** | `effects/mod.rs:5793` | **DomainCount always uses ctx.controller.** Allied Strategies says "lands they control" (target player), but DomainCount hardcodes the controller. **Fix:** Add `DomainCount { player: PlayerTarget }` or add a new `DomainCountOf` variant that takes a PlayerTarget. Update Allied Strategies to use `DomainCount { player: PlayerTarget::DeclaredTarget { index: 0 } }`. Territorial Maro stays with `Controller`. |
| 2 | **MEDIUM** | `rules/layers.rs:1472` | **CDA path uses base characteristics for DomainCount.** Blood Moon / Dryad effects (Layer 4) not reflected in Territorial Maro CDA. Documented as intentional (avoids recursion). Acceptable design but see Finding 3. **Fix:** No code change needed; see Finding 3 for doc fix. |
| 3 | **MEDIUM** | `cards/card_definition.rs:2097` | **Misleading doc comment.** Comment says "Uses layer-resolved characteristics" but the CDA path (layers.rs resolve_cda_amount) uses base characteristics. **Fix:** Update doc to: "Uses layer-resolved characteristics in the effect path (resolve_amount); in the CDA path (resolve_cda_amount), uses base characteristics to avoid recursion." |

## Card Definition Findings

| # | Severity | Card | Description |
|---|----------|------|-------------|
| 4 | **HIGH** | `allied_strategies.rs` | **Domain counts caster's lands, not target player's.** Oracle: "Target player draws a card for each basic land type among lands **they** control." The "they" = target player. Current def uses `DomainCount` which counts the controller's lands. **Fix:** After fixing Finding 1, update to `DomainCount { player: PlayerTarget::DeclaredTarget { index: 0 } }`. |
| 5 | **MEDIUM** | `bounty_of_skemfar.rs:13` | **Oracle text field diverges from actual oracle.** Def says "You may put a land card..." but real oracle says "You may put **up to one** land card...". Also omits "up to one" for Elf. **Fix:** Update oracle_text field to match real text: "...You may put up to one land card from among them onto the battlefield tapped and up to one Elf card from among them into your hand..." |

### Finding Details

#### Finding 1: DomainCount always uses ctx.controller

**Severity**: HIGH
**File**: `crates/engine/src/effects/mod.rs:5793`
**CR Rule**: N/A (this is a primitive design issue)
**Oracle**: Allied Strategies: "Target player draws a card for each basic land type among lands **they** control."
**Issue**: `EffectAmount::DomainCount` at line 5793 resolves `controller_id = ctx.controller`, meaning it always counts the caster's lands. Allied Strategies targets "target player" for both who draws AND whose lands are counted. If P1 casts Allied Strategies targeting P2, P2 should draw cards equal to the number of basic land types among lands **P2** controls -- not P1's lands. The current implementation gives the wrong count.
**Fix**: Change `DomainCount` to `DomainCount { player: PlayerTarget }`. Update resolve_amount to resolve the player target. Update resolve_cda_amount to use `controller` when `player == PlayerTarget::Controller`. Update Allied Strategies to `DomainCount { player: PlayerTarget::DeclaredTarget { index: 0 } }`. Update Territorial Maro to `DomainCount { player: PlayerTarget::Controller }`. Update hash.rs discriminant to hash the inner PlayerTarget. Update all existing `DomainCount` references (2 files: card defs + test).

#### Finding 2: CDA path uses base characteristics

**Severity**: MEDIUM
**File**: `crates/engine/src/rules/layers.rs:1472`
**CR Rule**: CR 305.6 -- "The basic land types are Plains, Island, Swamp, Mountain, and Forest."
**Issue**: `resolve_cda_amount` uses `obj.characteristics.subtypes` (base characteristics) instead of layer-resolved characteristics. This means Layer 4 effects like Blood Moon (makes non-basics into Mountains only) or Dryad of the Ilysian Grove (makes all lands every basic land type) are NOT reflected in Territorial Maro's P/T calculation. The comment documents this as intentional to avoid infinite recursion (CDA is evaluated during layer calculation). This is an acceptable design tradeoff -- partial layer resolution up to Layer 4 would be complex and fragile.
**Fix**: No code change. Add a comment in the CDA path noting this limitation explicitly: "// Limitation: Layer 4 type-changing effects (Blood Moon, Dryad) are not reflected here because resolve_cda_amount runs inside the layer loop. The resolve_amount path (effects/mod.rs) does use calculate_characteristics()."

#### Finding 3: Misleading doc comment on DomainCount enum variant

**Severity**: MEDIUM
**File**: `crates/engine/src/cards/card_definition.rs:2097`
**Issue**: Doc comment says "Uses layer-resolved characteristics so that effects like Dryad of the Ilysian Grove (Layer 4) are accounted for." This is only true in the `resolve_amount` path. In the CDA path (`resolve_cda_amount` in layers.rs), base characteristics are used. The doc is misleading.
**Fix**: Update doc comment to: "In the effect resolution path (resolve_amount), uses layer-resolved characteristics so that Layer 4 effects are accounted for. In the CDA path (resolve_cda_amount), uses base characteristics to avoid recursion during layer calculation."

#### Finding 4: Allied Strategies counts wrong player's lands

**Severity**: HIGH (duplicate of Finding 1, card def perspective)
**File**: `crates/engine/src/cards/defs/allied_strategies.rs:22`
**Oracle**: "Domain -- Target player draws a card for each basic land type among lands **they** control."
**Issue**: Uses `EffectAmount::DomainCount` which counts the caster's lands. Oracle says "they" = the target player.
**Fix**: After Finding 1 is resolved, update to `EffectAmount::DomainCount { player: PlayerTarget::DeclaredTarget { index: 0 } }`.

#### Finding 5: Bounty of Skemfar oracle_text diverges

**Severity**: MEDIUM
**File**: `crates/engine/src/cards/defs/bounty_of_skemfar.rs:13`
**Oracle**: "Reveal the top six cards of your library. You may put **up to one** land card from among them onto the battlefield tapped and **up to one** Elf card from among them into your hand. Put the rest on the bottom of your library in a random order."
**Issue**: The oracle_text field in the card def says "You may put a land card..." (missing "up to one") and omits "up to one Elf card" clause entirely. The functional approximation is documented in a NOTE, but the oracle_text field itself should match the real oracle text.
**Fix**: Update the oracle_text field to match the real oracle text exactly.

## Test Findings

| # | Severity | Description |
|---|----------|-------------|
| 6 | **MEDIUM** | **Missing test for layer-resolved domain.** Plan test 3 (`test_domain_count_dual_land`) was not implemented. No test exercises DomainCount with a continuous effect that changes land subtypes (e.g., Dryad of the Ilysian Grove making all lands every basic type). The header comment claims this is verified but no test exists. **Fix:** Add a test that registers a Layer 4 continuous effect adding all basic land subtypes to a single land, then verifies DomainCount = 5 via resolve_amount (effect path). This confirms the `calculate_characteristics()` call in effects/mod.rs works correctly. |

## CR Coverage Check

| CR Rule | Implemented? | Tested? | Notes |
|---------|-------------|---------|-------|
| CR 305.6 | Yes | Partial | DomainCount implemented; missing layer-resolved test; Allied Strategies player bug |
| CR 118.9 | Yes | Yes | CommanderFreeCast validation + cost override; 3 tests |
| CR 118.9a | Yes | No | Only one alt cost at a time; no explicit test (enforced by CastSpell.alt_cost being Option) |
| CR 118.9c | Yes | No | Mana cost unchanged; implicit in design |
| CR 118.9d | Yes | No | Additional costs still apply; no explicit test |
| CR 701.20 | Yes | Yes | Coiling Oracle RevealAndRoute; 2 tests |
| CR 604.3 | Yes | Yes | Territorial Maro CDA; 2 tests |
| CR 613.4a | Yes | Yes | CDA in Layer 7a (PtCda); tested via calculate_characteristics |

## Card Def Summary

| Card | Oracle Match | TODOs Remaining | Game State Correct | Notes |
|------|-------------|-----------------|-------------------|-------|
| allied_strategies | No | 0 | No | Domain counts caster's lands, not target player's (Finding 1/4) |
| territorial_maro | Yes | 0 | Yes | CDA P/T = 2 * domain_count via Sum. Base-chars-only CDA is acceptable limitation. |
| coiling_oracle | Yes | 0 | Yes | RevealAndRoute with land filter, correct destinations |
| bounty_of_skemfar | No | 0 | Partial | Oracle text field wrong; routes ALL lands not "up to one"; documented approximation |
| fierce_guardianship | Yes | 0 | Yes | CommanderFreeCast + CounterSpell noncreature |
| deadly_rollick | Yes | 0 | Yes | CommanderFreeCast + ExileObject creature |
| flawless_maneuver | Yes | 0 | Yes | CommanderFreeCast + ForEach indestructible until EOT |
