# Card Review: F-4 Session 4 Batch E

**Reviewed**: 2026-03-22
**Cards**: 12 (10 new/re-authored + 2 stale-comment cleanups)
**Findings**: 1 HIGH, 3 MEDIUM, 4 LOW

---

## Card 1: Bala Ged Recovery // Bala Ged Sanctuary
- **Oracle match**: YES (front face only, correct for MDFC)
- **Types match**: YES (Sorcery)
- **Mana cost match**: YES ({2}{G})
- **DSL correctness**: YES
- **Findings**:
  - F1 (LOW): No back face (Bala Ged Sanctuary, land that enters tapped) modeled. MDFC back faces are generally not implemented; consistent with project approach. No action needed.

## Card 2: Kabira Takedown // Kabira Plateau
- **Oracle match**: YES
- **Types match**: YES (Instant)
- **Mana cost match**: YES ({1}{W})
- **DSL correctness**: PARTIAL
- **Findings**:
  - F2 (LOW): TODO for TargetCreatureOrPlaneswalker is a valid DSL gap -- no such variant exists in TargetRequirement. Using TargetCreature is a reasonable approximation. Correctly documented.
  - F3 (LOW): `EffectAmount::PermanentCount` with `has_card_type: Some(CardType::Creature)` and `controller: PlayerTarget::Controller` is correct for "equal to the number of creatures you control."

## Card 3: Fell the Profane // Fell Mire
- **Oracle match**: YES (front face: "Destroy target creature or planeswalker.")
- **Types match**: YES (Instant)
- **Mana cost match**: YES ({2}{B}{B})
- **DSL correctness**: PARTIAL
- **Findings**:
  - F4 (LOW): Same TargetCreatureOrPlaneswalker gap as Kabira Takedown. Valid TODO. No action needed.

## Card 4: Consign // Oblivion
- **Oracle match**: YES (Consign half: "Return target nonland permanent to its owner's hand.")
- **Types match**: YES (Instant for Consign half)
- **Mana cost match**: YES ({1}{U} for Consign half)
- **DSL correctness**: YES
- **Findings**:
  - F5 (MEDIUM): Aftermath half (Oblivion: "{4}{B} — Target player discards two cards.") is not implemented. The comment says "DSL gap for graveyard-cast split cards" but Aftermath IS supported (AbilityDefinition::AltCastAbility / KeywordAbility::Aftermath, Batch 4). The Consign half alone is correct, but this TODO is stale -- Aftermath is expressible. **KI-3 candidate**: the claim that Aftermath is a DSL gap is incorrect; Aftermath was implemented in Batch 4.
  - F6: `TargetPermanentWithFilter` with `non_land: true` is correct (KI-1 compliant).
  - F7: `PlayerTarget::OwnerOf` for "its owner's hand" is correct for multiplayer (KI-11 compliant).

## Card 5: Sink into Stupor // Soporific Springs
- **Oracle match**: YES
- **Types match**: YES (Instant)
- **Mana cost match**: YES ({1}{U}{U})
- **DSL correctness**: PARTIAL
- **Findings**:
  - F8 (MEDIUM): TODO says "Should also target spells on the stack -- no combined target variant." This is a valid DSL gap (no TargetSpellOrNonlandPermanent). However, the card can only bounce permanents, not counter spells on the stack. This means the card is functionally incomplete -- it can't interact with spells at all. The TODO is correctly documented. Acceptable approximation.
  - F9: `TargetController::Opponent` is correct for "an opponent controls."
  - F10: `PlayerTarget::OwnerOf` is correct for "its owner's hand."

## Card 6: Sundering Eruption // Volcanic Fissure
- **Oracle match**: YES
- **Types match**: YES (Sorcery)
- **Mana cost match**: YES ({2}{R})
- **DSL correctness**: PARTIAL
- **Findings**:
  - F11 (MEDIUM): The "may search" is modeled as unconditional search. Per oracle and rulings, the controller MAY search (optional). The deterministic fallback comment is noted but this means the opponent is forced to search when they might not want to. Acceptable for now given no optional-search DSL primitive.
  - F12: The TODO for "creatures without flying can't block this turn" is a valid DSL gap -- mass blocking restrictions as a spell effect are not supported. Correctly documented.
  - F13: `ControllerOf(DeclaredTarget { index: 0 })` is correct for "its controller" in both SearchLibrary and Shuffle.
  - F14: `TargetLand` targeting is correct.

## Card 7: Decadent Dragon // Expensive Taste
- **Oracle match**: YES (front face: "Flying, trample\nWhenever Decadent Dragon attacks, create a Treasure token.")
- **Types match**: YES (Creature -- Dragon)
- **Mana cost match**: YES ({2}{R}{R})
- **P/T**: YES (4/4)
- **DSL correctness**: PARTIAL
- **Findings**:
  - F15 (HIGH): Adventure face (Expensive Taste: {2}{B} Instant -- "Exile the top two cards of target player's library face down. You may look at and play those cards for as long as they remain exiled, and you may spend mana as though it were mana of any color to cast them.") is not implemented and no `adventure_face` field is set. Adventure casting IS supported (PB-22 S7, `adventure_face` on CardDefinition). The comment says "Adventure half not implemented -- needs exile-play mechanic" which is partially valid (the exile-and-play-from-exile with mana-fixing is complex), but the `adventure_face` field should at minimum be populated with the face characteristics even if the effect is a TODO. **KI-3 partial**: Adventure infrastructure exists but the specific Expensive Taste effect (exile face-down + play from exile + any-color mana) is genuinely complex. The TODO is half-valid -- the adventure_face field should still be set.
  - F16: `WhenAttacks` trigger with `treasure_token_spec(1)` is correct.
  - F17: Flying + Trample keywords are correct.

## Card 8: Witch Enchanter // Witch-Blessed Meadow
- **Oracle match**: YES
- **Types match**: YES (Creature -- Human Warlock)
- **Mana cost match**: YES ({3}{W})
- **P/T**: YES (2/2)
- **DSL correctness**: YES
- **Findings**:
  - F18: `has_card_types: vec![CardType::Artifact, CardType::Enchantment]` with OR semantics is the correct pattern for "artifact or enchantment" (matches Bane of Progress, Takenuma, etc.).
  - F19: `TargetController::Opponent` is correct for "an opponent controls."
  - Clean card. No issues.

## Card 9: Scavenger Regent // Exude Toxin
- **Oracle match**: YES (front face: "Flying\nWard--Discard a card.")
- **Types match**: YES (Creature -- Dragon)
- **Mana cost match**: YES ({3}{B})
- **P/T**: YES (4/4)
- **DSL correctness**: PARTIAL
- **Findings**:
  - F20: TODO for Ward--Discard is a valid DSL gap. `Ward(u32)` only supports generic mana costs; non-mana ward costs (discard, pay life, sacrifice) have no DSL variant. The creature is castable with Flying but missing Ward, which is a partial implementation. However, Ward is defensive (protects the creature) so missing it makes the creature slightly weaker, not stronger -- no wrong game state produced. Acceptable per W5 policy.
  - F21: No Omen face (Exude Toxin) modeled. Omen is not supported in DSL (no omen_face equivalent). Consistent with project approach for unsupported dual-zone mechanics beyond Adventure.

## Card 10: Flawless Maneuver
- **Oracle match**: YES
- **Types match**: YES (Instant)
- **Mana cost match**: YES ({2}{W})
- **DSL correctness**: YES
- **Findings**:
  - F22: TODO for conditional free cast ("if you control a commander") is a valid DSL gap. No `Condition::ControlACommander` or conditional alt-cost mechanism exists. The main effect (indestructible until end of turn) is correctly implemented.
  - F23: ForEach/EachCreatureYouControl + ApplyContinuousEffect pattern matches Heroic Intervention exactly. Correct.
  - F24: `AddKeyword(KeywordAbility::Indestructible)` with `EffectLayer::Ability` and `EffectDuration::UntilEndOfTurn` is correct.

## Card 11: High Market (stale comment cleanup)
- **Oracle match**: YES
- **Types match**: YES (Land)
- **Mana cost match**: YES (None)
- **DSL correctness**: YES
- **Findings**: Clean. Header comment no longer has stale "(TODO)". Both abilities (tap for colorless, tap+sacrifice creature for 1 life) are correctly implemented.

## Card 12: Grim Backwoods (stale comment cleanup)
- **Oracle match**: YES
- **Types match**: YES (Land)
- **Mana cost match**: YES (None)
- **DSL correctness**: YES
- **Findings**: Clean. Header comment no longer has stale "(TODO)". Both abilities (tap for colorless, mana+tap+sacrifice for draw) are correctly implemented.

---

## Summary

### Finding Severity Counts
- **HIGH**: 1 (F15 -- Decadent Dragon missing adventure_face field)
- **MEDIUM**: 3 (F5 -- Consign stale Aftermath TODO; F8 -- Sink into Stupor spell targeting gap; F11 -- Sundering Eruption forced search)
- **LOW**: 4 (F1 -- Bala Ged no back face; F2/F4 -- TargetCreatureOrPlaneswalker gap x2; F20 -- Ward--Discard gap)

### Cards with issues
- **Decadent Dragon** (HIGH): adventure_face field should be populated even if effect is TODO
- **Consign** (MEDIUM): Aftermath TODO is stale -- Aftermath IS supported since Batch 4
- **Sink into Stupor** (MEDIUM): Can't target spells (valid gap, documented)
- **Sundering Eruption** (MEDIUM): "may search" modeled as forced search (valid gap, documented)
- **Kabira Takedown** (LOW): TargetCreature approximation for creature-or-planeswalker
- **Fell the Profane** (LOW): Same TargetCreature approximation
- **Bala Ged Recovery** (LOW): No MDFC back face
- **Scavenger Regent** (LOW): Ward--Discard gap, no Omen face

### Clean cards
- Witch Enchanter
- Flawless Maneuver
- High Market (cleanup verified)
- Grim Backwoods (cleanup verified)

### Action Items
1. **Decadent Dragon** (HIGH): Populate `adventure_face` with Expensive Taste characteristics (name, mana cost {2}{B}, types Instant -- Adventure, oracle text). Effect can remain as TODO/empty since exile-play-from-exile is complex.
2. **Consign** (MEDIUM/KI-3): Update comment to acknowledge Aftermath is supported. Ideally implement the Oblivion half using the Aftermath infrastructure from Batch 4 (discard 2 cards from target player).
