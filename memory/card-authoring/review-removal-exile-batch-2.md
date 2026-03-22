# Card Review: Removal/Exile Batch 2

**Reviewed**: 2026-03-22
**Cards**: 5
**Findings**: 1 HIGH, 1 MEDIUM, 1 LOW

## Card 1: Deathrite Shaman
- **Oracle match**: YES
- **Types match**: YES (Creature -- Elf Shaman, no supertypes needed)
- **Mana cost match**: YES (hybrid B/G)
- **P/T match**: YES (1/2)
- **DSL correctness**: YES
- **Findings**: None

All three activated abilities are correctly modeled:
- Ability 1: Tap, exile target land from graveyard, add any color -- correct.
- Ability 2: {B}+Tap, exile target instant/sorcery from graveyard, ForEach opponent loses 2 -- correct. `has_card_types` with OR semantics matches "instant or sorcery."
- Ability 3: {G}+Tap, exile target creature from graveyard, gain 2 life -- correct.
- `TargetCardInGraveyard` correctly used for "from a graveyard" (any player's).

## Card 2: Tear Asunder
- **Oracle match**: YES
- **Types match**: YES (Instant)
- **Mana cost match**: YES ({1}{G})
- **DSL correctness**: N/A (abilities empty)
- **Findings**: None

TODO claims DSL gap: Kicker changes the valid target set (artifact/enchantment vs nonland permanent). This is a genuine DSL gap -- there is no mechanism for kicker to alter the target requirement. W5 policy correctly applied with `abilities: vec![]`.

## Card 3: Return to Dust
- **Oracle match**: YES
- **Types match**: YES (Instant)
- **Mana cost match**: YES ({2}{W}{W})
- **DSL correctness**: N/A (abilities empty)
- **Findings**: None

TODO claims DSL gap: conditional second target gated on "during your main phase." This is a genuine DSL gap -- no conditional target slot mechanism exists. W5 policy correctly applied.

## Card 4: Reality Shift
- **Oracle match**: YES
- **Types match**: YES (Instant)
- **Mana cost match**: YES ({1}{U})
- **DSL correctness**: MOSTLY
- **Findings**:
  - F1 (HIGH): `PlayerTarget::ControllerOf` in Manifest effect falls back to `ctx.controller` (engine bug at effects/mod.rs:2549). In multiplayer, if you Reality Shift an opponent's creature, the manifest would go to the spell's controller instead of the exiled creature's controller. The card def expresses the correct intent (`ControllerOf(DeclaredTarget { index: 0 })`), but the engine's Manifest handler ignores this and uses the spell caster. This produces wrong game state in multiplayer. **However**: this is an engine-level bug, not a card def error. The card def is correct; the engine needs to resolve `ControllerOf` by looking up the target's controller (using LKI since the creature was exiled). Flagging as HIGH because it produces wrong game state.
  - F2 (MEDIUM): Related to F1 -- after the creature is exiled (first effect in Sequence), `DeclaredTarget { index: 0 }` refers to an object that no longer exists on the battlefield. `ControllerOf` would need LKI (last-known information) to determine who controlled it. The engine may not track this through the Sequence. This is also an engine concern rather than a card def issue.

## Card 5: Excise the Imperfect
- **Oracle match**: YES
- **Types match**: YES (Instant)
- **Mana cost match**: YES ({1}{W}{W})
- **DSL correctness**: N/A (abilities empty)
- **Findings**:
  - F3 (LOW): Comment on line 8 says "The exile effect is implemented" but `abilities: vec![]` means nothing is implemented. The comment is misleading -- it describes what would be partial, not what is. The code correctly follows W5 policy.

TODO claims DSL gap: `Effect::Incubate` does not exist in the DSL. Confirmed -- no `Incubate` variant in `Effect` enum. Genuine gap. W5 policy correctly applied (partial exile without the Incubator token would be wrong game state).

## Summary
- **Cards with issues**: Reality Shift (1 HIGH engine bug + 1 MEDIUM LKI concern), Excise the Imperfect (1 LOW misleading comment)
- **Clean cards**: Deathrite Shaman, Tear Asunder, Return to Dust
- **W5 policy correctly applied**: Tear Asunder, Return to Dust, Excise the Imperfect (all have genuine DSL gaps)
- **Engine bug identified**: `PlayerTarget::ControllerOf` is not resolved in the Manifest effect handler (effects/mod.rs:2549) -- falls back to `ctx.controller`. Affects Reality Shift and any card using `Manifest` with `ControllerOf`. This should be tracked as a separate engine fix.
