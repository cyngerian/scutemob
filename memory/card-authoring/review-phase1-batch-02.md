# Card Review: Phase 1 Batch 02 — ETB Tapped Dual/Multiplayer Lands

**Reviewed**: 2026-03-10
**Cards**: 5
**Findings**: 0 HIGH, 5 MEDIUM, 0 LOW

---

## Card 1: Spectator Seating
- **Oracle match**: YES
- **Types match**: YES (Land, no subtypes — correct)
- **Mana cost match**: YES (none)
- **DSL correctness**: NO
- **Findings**:
  - F1 (MEDIUM): ETB replacement is unconditional — always enters tapped. Oracle says "unless you have two or more opponents." The `EntersTapped` replacement has no condition check for opponent count. In Commander (4-player), this land should enter *untapped* by default. Missing TODO comment documenting this DSL gap (conditional ETB replacement). Ref: KI-9.

## Card 2: Clifftop Retreat
- **Oracle match**: YES
- **Types match**: YES (Land, no subtypes — correct)
- **Mana cost match**: YES (none)
- **DSL correctness**: NO
- **Findings**:
  - F1 (MEDIUM): ETB replacement is unconditional — always enters tapped. Oracle says "unless you control a Mountain or a Plains." The replacement should check the controller's battlefield for lands with Mountain or Plains subtypes. Missing TODO comment. Ref: KI-9.

## Card 3: Haunted Ridge
- **Oracle match**: YES
- **Types match**: YES (Land, no subtypes — correct)
- **Mana cost match**: YES (none)
- **DSL correctness**: NO
- **Findings**:
  - F1 (MEDIUM): ETB replacement is unconditional — always enters tapped. Oracle says "unless you control two or more other lands." The replacement should check if the controller has >= 2 other lands on the battlefield. Missing TODO comment. Ref: KI-9.

## Card 4: Rejuvenating Springs
- **Oracle match**: YES
- **Types match**: YES (Land, no subtypes — correct)
- **Mana cost match**: YES (none)
- **DSL correctness**: NO
- **Findings**:
  - F1 (MEDIUM): ETB replacement is unconditional — always enters tapped. Oracle says "unless you have two or more opponents." Same issue as Spectator Seating — in Commander this should enter untapped. Missing TODO comment. Ref: KI-9.

## Card 5: Hinterland Harbor
- **Oracle match**: YES
- **Types match**: YES (Land, no subtypes — correct)
- **Mana cost match**: YES (none)
- **DSL correctness**: NO
- **Findings**:
  - F1 (MEDIUM): ETB replacement is unconditional — always enters tapped. Oracle says "unless you control a Forest or an Island." The replacement should check the controller's battlefield for lands with Forest or Island subtypes. Missing TODO comment. Ref: KI-9.

---

## Summary

- **Cards with issues**: Spectator Seating, Clifftop Retreat, Haunted Ridge, Rejuvenating Springs, Hinterland Harbor (all 5)
- **Clean cards**: (none)

### Common Issue

All 5 cards share the same defect: the `EntersTapped` replacement effect is applied unconditionally with no "unless" condition. The DSL's `ReplacementModification::EntersTapped` does not support a condition predicate (e.g., opponent count >= 2, controlling a land with a specific subtype, controlling >= 2 other lands).

**Three distinct condition types are needed**:
1. **Opponent count** (Spectator Seating, Rejuvenating Springs): "unless you have two or more opponents" — untapped in Commander by default, tapped in 1v1.
2. **Land subtype check** (Clifftop Retreat, Hinterland Harbor): "unless you control a [Subtype] or a [Subtype]" — check battlefield for lands with matching basic land subtypes.
3. **Land count check** (Haunted Ridge): "unless you control two or more other lands" — count other lands on the controller's battlefield.

**Recommendation**: Each card should have a TODO comment on the replacement ability explaining the missing condition, per W5 policy (KI-9). The unconditional `EntersTapped` is a behavioral incorrectness — these lands will always enter tapped when they should often enter untapped. This is a known DSL gap (`shock_etb` / conditional ETB replacement) already tracked in the authoring worklist.

### Mana Verification (all correct)

| Card | Color 1 | mana_pool args | Color 2 | mana_pool args |
|------|---------|---------------|---------|---------------|
| Spectator Seating | R | (0,0,0,1,0,0) | W | (1,0,0,0,0,0) |
| Clifftop Retreat | R | (0,0,0,1,0,0) | W | (1,0,0,0,0,0) |
| Haunted Ridge | B | (0,0,1,0,0,0) | R | (0,0,0,1,0,0) |
| Rejuvenating Springs | G | (0,0,0,0,1,0) | U | (0,1,0,0,0,0) |
| Hinterland Harbor | G | (0,0,0,0,1,0) | U | (0,1,0,0,0,0) |
