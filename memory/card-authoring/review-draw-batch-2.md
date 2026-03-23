# Card Review: A-18 Draw Sessions S22-S23+S25

**Reviewed**: 2026-03-22
**Cards**: 19
**Findings**: 3 HIGH, 4 MEDIUM, 3 LOW

---

## Card 1: Aurelia, the Law Above
- **Oracle match**: YES
- **Types match**: YES (Legendary Creature -- Angel)
- **Mana cost match**: YES ({3}{R}{W})
- **P/T match**: YES (4/4)
- **DSL correctness**: YES
- **Findings**:
  - Keywords (Flying, Vigilance, Haste) correctly implemented.
  - TODO for "player attacks with 3+/5+ creatures" triggers is valid -- no such trigger condition exists in the DSL.
  - CLEAN

## Card 2: Sarkhan Unbroken
- **Oracle match**: YES
- **Types match**: YES (Legendary Planeswalker -- Sarkhan)
- **Mana cost match**: YES ({2}{G}{U}{R})
- **Loyalty match**: YES (4)
- **DSL correctness**: ISSUE
- **Findings**:
  - F1 (HIGH / KI-2): +1 ability partially implemented -- draws a card AND adds green mana. Oracle says "add one mana of any color" (player choice). The implementation hardcodes green mana via `mana_pool(0, 0, 0, 0, 1, 0)`. This produces wrong game state: always gives green instead of a chosen color. The +1 should use `Effect::Nothing` or the entire ability should note the approximation more clearly. W5 policy says partial impl producing wrong state should be `vec![]`, but since the draw is the more impactful part and there IS a TODO noting the gap, this is borderline. **Flag as HIGH because the mana addition is wrong (always green).**
  - F2 (LOW): -8 uses `Effect::Nothing` with TODO -- acceptable for "search any number of Dragons."
  - Token spec for -2 is correct (4/4 red Dragon with flying).

## Card 3: Caesar, Legion's Emperor
- **Oracle match**: YES
- **Types match**: YES (Legendary Creature -- Human Soldier)
- **Mana cost match**: YES ({1}{R}{W}{B})
- **P/T match**: YES (4/4)
- **DSL correctness**: YES
- **Findings**:
  - `abilities: vec![]` with TODOs for reflexive trigger + modal choice. Valid -- this is too complex for the DSL.
  - CLEAN

## Card 4: Falkenrath Pit Fighter
- **Oracle match**: YES
- **Types match**: YES (Creature -- Vampire Warrior)
- **Mana cost match**: YES ({R})
- **P/T match**: YES (2/1)
- **DSL correctness**: ISSUE
- **Findings**:
  - F3 (HIGH / KI-2): Uses `Cost::SacrificeSelf` but oracle says "Sacrifice a Vampire" (any Vampire you control, not necessarily self). This means the card works differently -- you can only sacrifice Pit Fighter itself instead of any Vampire. The activation condition ("only if an opponent lost life this turn") is also missing. The TODO acknowledges both issues but still implements a wrong version. Per W5 policy, this should be `abilities: vec![]` since the implementation produces wrong game state (wrong sacrifice target).
  - F4 (LOW): `activation_condition: None` when oracle requires "only if an opponent lost life this turn." Acknowledged in TODO.

## Card 5: Preacher of the Schism
- **Oracle match**: YES
- **Types match**: YES (Creature -- Vampire Cleric)
- **Mana cost match**: YES ({2}{B})
- **P/T match**: YES (2/4)
- **DSL correctness**: ISSUE
- **Findings**:
  - F5 (HIGH / KI-2): Implements the second trigger (draw + lose life) as unconditional `WhenAttacks`, but oracle requires "while you have the most life or are tied for most life." This triggers EVERY attack, drawing a card unconditionally. The first trigger (create Vampire token) is completely missing. Per W5 policy, a card that draws a card every attack when it should only do so conditionally produces wrong game state. Should be `abilities: vec![Keyword(Deathtouch)]` with TODOs only.

## Card 6: Ophidian Eye
- **Oracle match**: YES
- **Types match**: YES (Enchantment -- Aura)
- **Mana cost match**: YES ({2}{U})
- **DSL correctness**: YES
- **Findings**:
  - Flash and Enchant creature correctly implemented.
  - TODO for "enchanted creature deals damage to opponent" trigger is valid -- no per-enchanted-creature damage trigger exists in the DSL.
  - CLEAN

## Card 7: Ingenious Prodigy
- **Oracle match**: YES
- **Types match**: YES (Creature -- Human Wizard)
- **Mana cost match**: YES ({X}{U} -- X is implicit, blue: 1 is correct)
- **P/T match**: YES (0/1)
- **DSL correctness**: MOSTLY OK
- **Findings**:
  - F6 (MEDIUM): Upkeep trigger is implemented but oracle says "you may remove a counter. If you do, draw a card." The "may" and "if you do" conditional is not represented -- the implementation always removes a counter and draws. This is an approximation that could matter (forced counter removal vs optional). The intervening_if correctly checks for counters, which is good.
  - TODO for "enters with X +1/+1 counters" is valid.
  - Skulk keyword correct.

## Card 8: Vampire Gourmand
- **Oracle match**: YES
- **Types match**: YES (Creature -- Vampire)
- **Mana cost match**: YES ({1}{B})
- **P/T match**: YES (2/2)
- **DSL correctness**: ISSUE
- **Findings**:
  - F7 (MEDIUM / KI-2): Implemented as unconditional "draw on attack" but oracle requires sacrificing another creature first. The "can't be blocked" part is also missing. Drawing a card every attack without requiring a sacrifice is wrong game state. Per W5 policy, should be `abilities: vec![]` with TODO.

## Card 9: Tandem Lookout
- **Oracle match**: YES
- **Types match**: YES (Creature -- Human Scout)
- **Mana cost match**: YES ({2}{U})
- **P/T match**: YES (2/1)
- **DSL correctness**: YES
- **Findings**:
  - Soulbond keyword correctly present.
  - TODO for soulbond grant (triggered ability grant to paired creature) is valid -- SoulbondGrant lacks triggered ability grants.
  - CLEAN

## Card 10: Kaito Shizuki
- **Oracle match**: YES (oracle_text uses Kaito Shizuki name correctly)
- **Types match**: YES (Legendary Planeswalker -- Kaito)
- **Mana cost match**: YES ({1}{U}{B})
- **Loyalty match**: YES (3)
- **DSL correctness**: ISSUE
- **Findings**:
  - F8 (MEDIUM): +1 ability only draws a card but oracle says "Draw a card. Then discard a card unless you attacked this turn." The conditional discard is missing. The TODO doesn't mention this -- only the phase-out end step trigger is noted. The +1 is always-upside (draw without potential discard), which is wrong game state.
  - -2 token is missing "can't be blocked" ability on the Ninja token. The token has no keywords for unblockability. TODO would be appropriate since TokenSpec can't express "can't be blocked" as a static ability (it's not a keyword).
  - -7 emblem is omitted with TODO -- valid, emblem with complex trigger + search is beyond DSL.
  - Phase-out TODO is valid.

## Card 11: Zurgo Stormrender
- **Oracle match**: YES
- **Types match**: YES (Legendary Creature -- Orc Warrior)
- **Mana cost match**: YES ({R}{W}{B})
- **P/T match**: YES (3/3)
- **DSL correctness**: PARTIAL
- **Findings**:
  - Mobilize 1 token creation is correctly implemented (1/1 red Warrior, tapped and attacking).
  - TODO for "sacrifice at beginning of next end step" delayed trigger is valid.
  - TODO for "whenever a creature token you control leaves" is valid -- no such trigger condition.
  - No wrong game state from what's implemented (token creation on attack is correct behavior).
  - CLEAN (TODOs are valid)

## Card 12: Dusk Legion Duelist
- **Oracle match**: YES
- **Types match**: YES (Creature -- Vampire Soldier)
- **Mana cost match**: YES ({1}{W})
- **P/T match**: YES (2/2)
- **DSL correctness**: YES
- **Findings**:
  - Vigilance correctly implemented.
  - TODO for "+1/+1 counter placed trigger" + "once each turn" is valid -- no such trigger condition.
  - CLEAN

## Card 13: Street Wraith
- **Oracle match**: YES
- **Types match**: YES (Creature -- Wraith)
- **Mana cost match**: YES ({3}{B}{B})
- **P/T match**: YES (3/4)
- **DSL correctness**: ISSUE
- **Findings**:
  - F9 (MEDIUM / KI-2): Has BOTH `Keyword(Cycling)` AND `Cycling { cost: ManaCost::default() }`. The keyword marker is redundant when the AbilityDefinition::Cycling is present. More importantly, the cycling cost is `ManaCost::default()` (free mana cycling) but oracle says "Pay 2 life" -- this is life payment, not mana. Free cycling is wrong game state (cycles for free instead of costing 2 life). Should be `abilities: vec![Keyword(Landwalk(...)), Keyword(Cycling)]` with a TODO explaining the life-cost cycling gap, OR remove the `Cycling { cost }` entirely.
  - Swampwalk is correctly implemented.

## Card 14: Nezahal, Primal Tide
- **Oracle match**: YES
- **Types match**: YES (Legendary Creature -- Elder Dinosaur)
- **Mana cost match**: YES ({5}{U}{U})
- **P/T match**: YES (7/7)
- **DSL correctness**: PARTIAL
- **Findings**:
  - The triggered ability (opponent casts noncreature spell -> draw) is overbroad: triggers on ALL opponent spells, not just noncreature. The TODO acknowledges this. This is wrong game state but the approximation is documented.
  - TODOs for "can't be countered," "no max hand size," and "discard 3: exile + delayed return" are all valid DSL gaps.
  - LOW: The overbroad trigger should ideally be removed per W5 policy since it draws cards when it shouldn't (on creature spells too).

## Card 15: Kaito, Dancing Shadow
- **Oracle match**: YES
- **Types match**: YES (Legendary Planeswalker -- Kaito)
- **Mana cost match**: YES ({2}{U}{B})
- **Loyalty match**: YES (3)
- **DSL correctness**: PARTIAL
- **Findings**:
  - +1 uses `Effect::Nothing` with TODO -- valid, "can't attack or block until your next turn" is complex.
  - 0 (draw a card) is correctly implemented.
  - -2 token is correctly colorless Drone artifact creature with deathtouch. TODO for LTB trigger on token is valid (TokenSpec can't express triggered abilities).
  - Combat damage bounce + double loyalty activation TODO is valid.
  - CLEAN (TODOs are valid, no wrong game state)

## Card 16: Rummaging Goblin
- **Oracle match**: YES
- **Types match**: YES (Creature -- Goblin Rogue)
- **Mana cost match**: YES ({2}{R})
- **P/T match**: YES (1/1)
- **DSL correctness**: YES
- **Findings**:
  - Activated ability (tap + discard -> draw) is perfectly implemented.
  - CLEAN

## Card 17: Chivalric Alliance
- **Oracle match**: YES
- **Types match**: YES (Enchantment)
- **Mana cost match**: YES ({1}{W})
- **DSL correctness**: YES
- **Findings**:
  - TODO for "attack with 2+ creatures" trigger is valid.
  - Activated ability ({2} + discard -> create Knight token) is correctly implemented. Token is white/blue 2/2 Knight with vigilance -- matches oracle.
  - CLEAN

## Card 18: Grim Haruspex
- **Oracle match**: YES
- **Types match**: YES (Creature -- Human Wizard)
- **Mana cost match**: YES ({2}{B})
- **P/T match**: YES (3/2)
- **DSL correctness**: ISSUE
- **Findings**:
  - Morph keyword + Morph cost correctly implemented ({B}).
  - F10 (MEDIUM / KI-9): Uses `WheneverCreatureDies` for "another nontoken creature you control dies." The trigger is overbroad -- fires on ALL creature deaths (opponents' creatures, tokens, and self). The TODO acknowledges this. Per KI-9 policy, overbroad death triggers should use `abilities: vec![Keyword(Morph), Morph { cost }]` with TODO for the trigger. The current implementation draws cards on opponent creature deaths too.

## Card 19: Sea-Dasher Octopus
- **Oracle match**: YES
- **Types match**: YES (Creature -- Octopus)
- **Mana cost match**: YES ({1}{U}{U})
- **P/T match**: YES (2/2)
- **DSL correctness**: YES
- **Findings**:
  - Mutate keyword + MutateCost ({1}{U}) correctly implemented.
  - Flash keyword correctly present.
  - Triggered ability (combat damage to player -> draw) correctly uses `WhenDealsCombatDamageToPlayer`.
  - CLEAN

---

## Summary

### Cards with issues:
- **Sarkhan Unbroken** (HIGH): +1 hardcodes green mana instead of any-color choice
- **Falkenrath Pit Fighter** (HIGH): SacrificeSelf instead of "Sacrifice a Vampire" + missing activation condition -> wrong game state
- **Preacher of the Schism** (HIGH): Unconditional draw-on-attack without life-total condition -> wrong game state
- **Kaito Shizuki** (MEDIUM): +1 missing conditional discard; -2 token missing "can't be blocked"
- **Ingenious Prodigy** (MEDIUM): Upkeep trigger is mandatory instead of optional (may)
- **Vampire Gourmand** (MEDIUM): Unconditional draw-on-attack without sacrifice requirement -> wrong game state
- **Street Wraith** (MEDIUM): Free cycling instead of 2-life cycling + redundant Keyword(Cycling)
- **Grim Haruspex** (MEDIUM / KI-9): Overbroad WheneverCreatureDies trigger
- **Nezahal, Primal Tide** (LOW): Overbroad opponent-casts-spell trigger (all spells, not noncreature only)

### Clean cards (10):
- Aurelia, the Law Above
- Caesar, Legion's Emperor
- Ophidian Eye
- Tandem Lookout
- Zurgo Stormrender
- Dusk Legion Duelist
- Kaito, Dancing Shadow
- Rummaging Goblin
- Chivalric Alliance
- Sea-Dasher Octopus

### Recommended fixes:
1. **Falkenrath Pit Fighter**: Replace abilities with `vec![]` + TODO (SacrificeSelf is wrong target)
2. **Preacher of the Schism**: Remove the triggered ability, keep only `Keyword(Deathtouch)` + TODOs
3. **Vampire Gourmand**: Replace abilities with `vec![]` + TODO (free draw without sac is wrong)
4. **Street Wraith**: Remove `Cycling { cost: ManaCost::default() }`, keep only `Keyword(Cycling)` + TODO for life-cost cycling
5. **Grim Haruspex**: Remove overbroad triggered ability, keep Morph keyword + cost + TODO
6. **Sarkhan Unbroken**: Either remove the AddMana from +1 sequence or document the green-only approximation more explicitly
7. **Kaito Shizuki**: Add TODO noting +1 is missing conditional discard; add TODO for -2 token missing "can't be blocked"
8. **Nezahal, Primal Tide**: Remove overbroad trigger, add TODO (draws on creature spells too)
