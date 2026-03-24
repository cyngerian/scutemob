# Card Review: removal-damage-each batch 1

**Reviewed**: 2026-03-22
**Cards**: 5
**Findings**: 1 HIGH, 2 MEDIUM, 1 LOW

## Card 1: Witty Roastmaster
- **Oracle match**: YES
- **Types match**: YES (Creature -- Devil Citizen)
- **Mana cost match**: YES ({2}{R})
- **P/T match**: YES (3/2)
- **DSL correctness**: YES
- **Findings**: CLEAN
- Notes: `WheneverCreatureEntersBattlefield` with `controller: TargetController::You` correctly models "another creature you control enters." The `exclude_self` flag is automatically set to `true` in `enrich_spec_from_def` for all `WheneverCreatureEntersBattlefield` triggers, so "another" is correctly handled. `ForEach::EachOpponent` with `DealDamage` correctly targets each opponent.

## Card 2: Syr Konrad, the Grim
- **Oracle match**: YES
- **Types match**: YES (Legendary Creature -- Human Knight)
- **Mana cost match**: YES ({3}{B}{B})
- **P/T match**: YES (5/4)
- **DSL correctness**: NO
- **Findings**:
  - F1 (MEDIUM / KI-9): `WheneverCreatureDies` is overbroad -- oracle says "another creature dies" but `WheneverCreatureDies` has no self-exclusion filter. Furthermore, per engine comment at abilities.rs:5775, `WheneverCreatureDies` triggers are NOT wired in `enrich_spec_from_def` and never actually fire at runtime. The trigger is a silent no-op. The TODO comments at lines 23-26 correctly document the 3-part trigger gap (non-battlefield graveyard entry, graveyard departure). These are genuine DSL gaps.
  - F2 (LOW): Missing activated ability "{1}{B}: Each player mills a card." The TODO at line 39 correctly notes this gap. `Effect::Mill` exists but wiring it with `ForEach::EachPlayer` for an activated ability should be expressible. However, the mill effect on each player (not just self) may need verification of DSL support.

## Card 3: Purphoros, God of the Forge
- **Oracle match**: YES
- **Types match**: YES (Legendary Enchantment Creature -- God)
- **Mana cost match**: YES ({3}{R})
- **P/T match**: YES (6/5)
- **DSL correctness**: YES (for implemented portions)
- **Findings**:
  - F3 (MEDIUM / KI-2): W5 partial implementation concern. Purphoros has 4 abilities: Indestructible (implemented), devotion-based type loss (TODO -- genuine gap, requires devotion count + Layer 4 type removal), ETB damage trigger (implemented correctly), and pump activated ability (TODO). The devotion check is the critical missing piece -- without it, Purphoros is always a 6/5 indestructible creature, which is significantly stronger than the real card (which is usually just an indestructible enchantment). This produces wrong game state: Purphoros attacks/blocks as a creature when it shouldn't be one in most board states. Per W5 policy, should be `abilities: vec![]` since the partial implementation makes the card strictly better than intended. However, the damage trigger alone (without being a creature) is still correct behavior -- Purphoros deals damage even when not a creature. This is a judgment call.
  - Notes: The ETB trigger uses `WheneverCreatureEntersBattlefield` with `controller: TargetController::You`, which is correct. `exclude_self` is automatically applied. `ForEach::EachOpponent` with `DealDamage` amount 2 matches oracle.

## Card 4: Crackling Doom
- **Oracle match**: YES
- **Types match**: YES (Instant)
- **Mana cost match**: YES ({R}{W}{B})
- **DSL correctness**: NO
- **Findings**:
  - F4 (HIGH / KI-2): W5 partial implementation produces wrong game state. The card implements only "deals 2 damage to each opponent" but omits the critical "Each opponent sacrifices a creature with the greatest power among creatures that player controls." The sacrifice effect is the primary purpose of this card -- it is premier targeted-sacrifice removal in Mardu. Implementing only the 2 damage makes this a drastically underpowered version that still resolves and produces an incorrect game outcome (opponents take 2 but keep their best creature). Per W5 policy, a card that does the wrong thing is worse than `abilities: vec![]`. The TODO at line 15 correctly identifies the gap (per-opponent greatest-power filtering + forced sacrifice). Should be `abilities: vec![]` until the sacrifice portion can be implemented.

## Card 5: Sabotender
- **Oracle match**: YES
- **Types match**: YES (Creature -- Plant)
- **Mana cost match**: YES ({1}{R})
- **P/T match**: YES (2/1)
- **DSL correctness**: YES
- **Findings**: CLEAN
- Notes: `Reach` keyword present. Landfall trigger uses `WheneverPermanentEntersBattlefield` with `has_card_type: Some(CardType::Land)` and `controller: TargetController::You`, which correctly models "Whenever a land you control enters." `ForEach::EachOpponent` with `DealDamage` amount 1 matches oracle.

## Summary
- **Cards with issues**: Syr Konrad, the Grim (1 MEDIUM, 1 LOW); Purphoros, God of the Forge (1 MEDIUM); Crackling Doom (1 HIGH)
- **Clean cards**: Witty Roastmaster, Sabotender

### Issue Index
| ID | Sev | Card | Issue |
|----|-----|------|-------|
| F1 | MEDIUM | Syr Konrad | WheneverCreatureDies overbroad + never fires (silent no-op) |
| F2 | LOW | Syr Konrad | Missing "{1}{B}: Each player mills a card" activated ability |
| F3 | MEDIUM | Purphoros | Missing devotion type-loss makes card always a creature (too strong) |
| F4 | HIGH | Crackling Doom | Partial impl (damage only, no sacrifice) -- should be vec![] per W5 |
