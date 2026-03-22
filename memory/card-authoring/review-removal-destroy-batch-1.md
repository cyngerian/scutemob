# Card Review: Removal/Destroy Batch 1

**Reviewed**: 2026-03-22
**Cards**: 5
**Findings**: 3 HIGH, 2 MEDIUM, 0 LOW

## Card 1: Abrade
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES
- **DSL correctness**: YES
- **Findings**: None -- clean card.

## Card 2: Pyroblast
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES
- **DSL correctness**: NO
- **Findings**:
  - F1 (HIGH): Pyroblast's target filters incorrectly restrict targeting to blue spells/permanents. Per 2016-06-08 ruling: "Pyroblast can target any spell or permanent, not just a blue one. It checks the color of the target only on resolution." The `TargetRequirement` entries should be `TargetSpell` and `TargetPermanent` (no color filter). The "if it's blue" check is an on-resolution condition, not a targeting restriction. The current implementation prevents casting Pyroblast when there are no blue targets, which is incorrect.

## Card 3: Terminate
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES
- **DSL correctness**: PARTIAL
- **Findings**:
  - F2 (MEDIUM): "It can't be regenerated" clause is not modeled. The card uses plain `DestroyPermanent` without any anti-regeneration flag. Regeneration is a supported mechanic (P2 ability). If the engine enforces regeneration shields, this card would allow regeneration when it should not. Severity is MEDIUM because regeneration is rarely relevant in Commander but the behavior is technically wrong when it matters.

## Card 4: Nature's Claim
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES
- **DSL correctness**: NO
- **Findings**:
  - F3 (HIGH / KI-3): TODO claims `PlayerTarget::TargetController` is needed and does not exist. This is a stale DSL gap. `PlayerTarget::ControllerOf(Box<EffectTarget>)` exists (card_definition.rs:1601). The GainLife effect should use `player: PlayerTarget::ControllerOf(Box::new(EffectTarget::DeclaredTarget { index: 0 }))` instead of `PlayerTarget::Controller`. Current implementation gives life to the spell's caster instead of the destroyed permanent's controller -- wrong in multiplayer.

## Card 5: Stroke of Midnight
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES
- **DSL correctness**: NO
- **Findings**:
  - F4 (HIGH / KI-3): Same stale DSL gap as Nature's Claim. TODO claims `PlayerTarget::TargetController` is needed, but `PlayerTarget::ControllerOf(Box<EffectTarget>)` already exists. The `CreateToken` effect should use `PlayerTarget::ControllerOf(Box::new(EffectTarget::DeclaredTarget { index: 0 }))` so the token is created under the destroyed permanent's controller, not the spell's caster. Current implementation gives the token to the wrong player in multiplayer.
  - F5 (MEDIUM): `CreateToken` does not specify a `controller` / `player` field at all -- it appears to default to the spell's controller. Need to verify whether `CreateToken` supports a player target field for creating tokens under a different player's control. If not, this is a genuine DSL gap (not the `PlayerTarget` variant, but the `CreateToken` effect lacking a player parameter).

## Summary
- **Cards with issues**: Pyroblast (1 HIGH), Terminate (1 MEDIUM), Nature's Claim (1 HIGH), Stroke of Midnight (1 HIGH + 1 MEDIUM)
- **Clean cards**: Abrade

### Fix Priority
1. **Pyroblast** (F1): Remove color filters from `TargetRequirement` entries. Use `TargetSpell` and `TargetPermanent`. The on-resolution blue check may need a `Conditional` wrapper or `intervening_if` -- verify engine support for on-resolution color checks in modal spells.
2. **Nature's Claim** (F3): Replace `PlayerTarget::Controller` with `PlayerTarget::ControllerOf(Box::new(EffectTarget::DeclaredTarget { index: 0 }))`.
3. **Stroke of Midnight** (F4): Same PlayerTarget fix. Also verify whether `CreateToken` supports creating tokens under a specific player (F5).
4. **Terminate** (F2): Add anti-regeneration flag to `DestroyPermanent` if the engine supports it; otherwise document as known limitation.
