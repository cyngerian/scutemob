# Stress Test Scenarios

**Date**: 2026-03-27
**Purpose**: Curated list of card combinations known to stress MTG engine implementations.
These target the gap identified in `docs/mtg-engine-runtime-integrity.md` — "legal-but-wrong"
states where the engine is internally consistent but applies an incorrect rule. The runtime
invariant checker cannot catch these; only scenario-specific tests can.

**Format**: Each scenario names the cards involved, the expected correct behavior with CR
citation, and the engine subsystems being stressed.

**Status**: PROPOSED — scenarios need to be converted to game scripts as capacity allows.
Priority order reflects likelihood of occurring in real Commander games.

---

## Priority 1: Ability Removal Interactions

These test the layer bypass bug class (`docs/audits/layer-bypass-audit.md`). Should be
written alongside the layer bypass fixes in M10.

### S-01: Humility + Thalia, Guardian of Thraben
- **Cards**: Humility, Thalia Guardian of Thraben
- **Expected**: Humility removes all creature abilities (Layer 6). Thalia's cost-taxing
  static ability is removed. Noncreature spells cost normally.
- **CR**: 604.1 (static abilities), 613.1f (Layer 6 removes abilities)
- **Subsystems**: Layer system, casting cost calculation
- **Current bug**: casting.rs reads `spell_cost_modifiers` from static CardDefinition

### S-02: Blood Moon + Urza's Saga
- **Cards**: Blood Moon, Urza's Saga
- **Expected**: Urza's Saga loses land types and abilities, gains Mountain. Saga subtype
  remains (enchantment subtype, not land type). No chapter abilities → SBA does NOT
  sacrifice (CR 714.4: "with one or more chapter abilities"). Saga sits as a Mountain.
  Previously-granted abilities (from lore counters before Blood Moon) persist.
- **CR**: 305.7 (Blood Moon type change), 714.4 (Saga sacrifice SBA), 613.1d (Layer 4)
- **Subsystems**: Layer system, SBA saga check, type changing effects

### S-03: Dress Down + Exploit creature
- **Cards**: Dress Down, Sidisi Undead Vizier
- **Expected**: Dress Down removes all creature abilities until end of turn (Layer 6).
  Sidisi's Exploit ability is gone — no Exploit trigger on ETB.
- **CR**: 702.120a (Exploit), 613.1f (Layer 6)
- **Subsystems**: Layer system, ETB trigger generation, keyword counting

### S-04: Humility + Flanking creature in combat
- **Cards**: Humility, Cavalry Master (double flanking)
- **Expected**: Humility removes Flanking. No -1/-1 triggers when blocked.
- **CR**: 702.25 (Flanking), 613.1f
- **Subsystems**: Layer system, combat trigger generation, keyword counting

### S-05: Blood Moon + nonbasic land with cost reduction
- **Cards**: Blood Moon, any nonbasic land with a static ability (e.g., Urborg Tomb of
  Yawgmoth with cost reducers, or a land that grants abilities)
- **Expected**: Land loses all abilities, becomes a Mountain.
- **CR**: 305.7, 613.1
- **Subsystems**: Layer system, land type interaction, ability removal

---

## Priority 2: Layer System Ordering

These test that continuous effects are applied in the correct layer order with correct
dependency resolution.

### S-06: Humility + Opalescence
- **Cards**: Humility, Opalescence ("Each other non-Aura enchantment is a creature with
  P/T equal to its mana value")
- **Expected**: Dependency — Humility removes abilities, Opalescence makes enchantments
  into creatures. Timestamp order matters. If Opalescence entered first: Humility makes
  all creatures 1/1 and removes abilities, but Humility itself is a creature (via
  Opalescence) so it also loses its own ability. Circular dependency resolved by timestamp.
- **CR**: 613.8 (dependency), 613.7 (timestamp)
- **Subsystems**: Layer system dependency detection, circular dependency resolution

### S-07: Copying a creature with continuous effects
- **Cards**: Clone, any creature with a +1/+1 counter and an anthem effect
- **Expected**: Clone copies the base creature (Layer 1). Counters are not copiable
  (CR 707.2). Anthem effects apply independently.
- **CR**: 707.2 (copiable values), 613.1a (Layer 1)
- **Subsystems**: Copy effects, layer system, counter handling

### S-08: Multiple anthem effects with different timestamps
- **Cards**: 2x Glorious Anthem ("Creatures you control get +1/+1")
- **Expected**: Both apply at Layer 7c. Order doesn't matter (additive). Total +2/+2.
- **CR**: 613.1g (Layer 7c)
- **Subsystems**: Layer system, continuous effect stacking

### S-09: Type-changing effect + anthem interaction
- **Cards**: Humility, Kormus Bell ("Swamps are 1/1 creatures"), Urborg Tomb of Yawgmoth
- **Expected**: Urborg makes all lands Swamps. Kormus Bell makes Swamps into creatures.
  Humility makes all creatures 1/1 and removes abilities. All lands are now 1/1 creatures
  with no abilities (including mana abilities — they can't tap for mana).
- **CR**: 613.1 (layer ordering), 305.7
- **Subsystems**: Layer system, type changing, ability removal interaction

### S-10: P/T setting vs modifying
- **Cards**: Godhead of Awe ("Other creatures are 1/1"), any creature with +1/+1 counters
- **Expected**: Godhead sets base P/T to 1/1 (Layer 7b). +1/+1 counters apply after
  (Layer 7c). Creature with 3 +1/+1 counters is 4/4, not 1/1.
- **CR**: 613.4b (Layer 7b before 7c)
- **Subsystems**: Layer system sublayer ordering

---

## Priority 3: Zone Change and Identity

These test CR 400.7 (new object on zone change) and LKI.

### S-11: Swords to Plowshares — LKI power reference
- **Cards**: Swords to Plowshares, any creature with modified power (e.g., pumped by
  Giant Growth)
- **Expected**: Creature is exiled. Controller gains life equal to the creature's power
  as it last existed on the battlefield (after Giant Growth, not base power).
- **CR**: 608.2g (LKI), 613.1
- **Subsystems**: Effect execution, LKI, layer-resolved power

### S-12: Oblivion Ring chain
- **Cards**: 3x Oblivion Ring
- **Expected**: O-Ring A exiles permanent X. O-Ring B exiles O-Ring A. X returns (A left
  battlefield). O-Ring C exiles O-Ring B. O-Ring A returns, triggers, exiles something new.
  If O-Ring C is destroyed, B returns, which triggers to exile A, which returns X again.
- **CR**: 400.7 (new object), 610.3 (one-shot LTB effect)
- **Subsystems**: Zone change identity, trigger queuing, LKI

### S-13: Flicker + ETB trigger ordering
- **Cards**: Restoration Angel, any creature with an ETB trigger
- **Expected**: Restoration Angel ETB exiles a creature, then returns it immediately.
  The returned creature is a new object (CR 400.7). Its ETB trigger fires. Angel's trigger
  and returned creature's trigger go on stack in APNAP order.
- **CR**: 400.7, 603.3b (APNAP)
- **Subsystems**: Zone change identity, trigger ordering, ETB queueing

### S-14: Creature dies then referenced
- **Cards**: any sacrifice outlet, any creature, any "when a creature dies" trigger that
  references the dead creature's characteristics
- **Expected**: Dead creature's power/toughness/abilities are read from LKI (layer-resolved
  state as it last existed on the battlefield), not from the new graveyard object.
- **CR**: 608.2g
- **Subsystems**: LKI, trigger resolution

---

## Priority 4: Replacement Effects

### S-15: Multiple replacement effects on same event
- **Cards**: Anointed Procession ("If an effect would create tokens, create twice that
  many"), Doubling Season, any token creator
- **Expected**: Affected player chooses which replacement to apply first. Both apply.
  1 token → 2 → 4 (or 1 → 2 → 4, same result for doubling).
- **CR**: 616.1 (player chooses order), 614.5
- **Subsystems**: Replacement effect ordering, `Command::OrderReplacements`

### S-16: Replacement effect + prevention interaction
- **Cards**: Furnace of Rath ("damage is doubled"), any damage prevention (Fog, protection)
- **Expected**: Replacement effects apply in order chosen by affected player. If doubling
  first: 3 → 6, then prevent 6. If prevention first: prevent 3, doubled amount is 0.
- **CR**: 615.1 (prevention), 614.1 (replacement order)
- **Subsystems**: Damage replacement, damage prevention, ordering

### S-17: ETB replacement — enters tapped + ETB trigger
- **Cards**: Thalia Heretic Cathar ("Creatures opponents control enter tapped"), any
  creature with an ETB trigger
- **Expected**: Creature enters tapped (replacement effect modifies how it enters).
  ETB trigger still fires (replacement doesn't prevent entering, just modifies it).
- **CR**: 614.1c (replacement modifies event, doesn't prevent)
- **Subsystems**: ETB replacement, trigger queueing, replacement vs trigger distinction

---

## Priority 5: Commander-Specific

### S-18: Commander tax + cost reduction interaction
- **Cards**: Any commander cast multiple times, Thalia (cost increase), Goblin Warchief
  (cost reduction for Goblins)
- **Expected**: Commander tax is additional cost (CR 903.8). Thalia adds {1}. Warchief
  reduces {1}. All additive. Generic component cannot go below 0.
- **CR**: 903.8, 601.2f
- **Subsystems**: Casting cost calculation, commander tax, cost modifier stacking

### S-19: Commander damage across zone changes
- **Cards**: Any commander that dies and is recast
- **Expected**: Commander damage tracking persists per-commander across recasts (CR 903.10a).
  21 total commander combat damage from the same commander = loss, regardless of recasts.
- **CR**: 903.10a, 704.6c
- **Subsystems**: Commander damage tracking, zone change identity

### S-20: Partner commanders + commander tax
- **Cards**: Two partner commanders
- **Expected**: Each partner tracks its own commander tax independently (CR 903.8).
  Casting partner A twice then partner B once = A costs {4} more, B costs {2} more.
- **CR**: 903.8, 702.124
- **Subsystems**: Commander tax per-commander tracking

---

## Priority 6: Multiplayer-Specific

### S-21: APNAP trigger ordering with 4 players
- **Cards**: 4 players each with a "whenever a creature enters" trigger, one creature entering
- **Expected**: Active player's triggers first, then next in turn order, etc. (CR 603.3b).
  Within each player's triggers, that player chooses the order.
- **CR**: 603.3b
- **Subsystems**: Trigger flushing, APNAP sort

### S-22: Nine Lives APNAP — multiplayer simultaneous "lose the game" triggers
- **Cards**: 4x Nine Lives (one per player, given via Trickster God's Heist or similar),
  Cyclone Summoner or any mass bounce
- **Setup**: 4-player Commander. Each player controls a Nine Lives with enough counters
  to trigger "lose the game" on leaving. Active player bounces everything.
- **Expected**: All four Nine Lives leave simultaneously. Four "lose the game" triggers.
  APNAP order: active player's trigger goes on stack first (bottom), then next in turn
  order, then next, then last. Stack resolves top-down. The player **furthest from the
  active player in turn order** has their trigger on top and loses first. Game ends
  immediately — remaining triggers never resolve.
- **Why this matters**: In 2-player, APNAP is simple (active loses last). In 4-player,
  the ordering becomes turn-order-dependent. A player bouncing everything on their own
  turn survives because their trigger is on the bottom. The player to their "left"
  (last in APNAP) loses first. Getting this wrong means the wrong player dies.
- **Real-world precedent**: This exact interaction (with 2 players) was reported in
  MTG Arena ranked play — the non-active player lost despite the active player having
  more "lose the game" triggers, purely due to APNAP stack ordering.
- **CR**: 603.3b (APNAP ordering), 104.4a (player loses when "lose the game" resolves)
- **Subsystems**: Trigger flushing, APNAP sort, multiplayer turn order, game termination

### S-23: "Each opponent" in 4-player game after elimination (was S-22)
- **Cards**: Any "each opponent loses 1 life" effect, 4 players, one eliminated
- **Expected**: Effect targets remaining 2 opponents, not the eliminated player.
- **CR**: 104.3a (opponent = other player in the game)
- **Subsystems**: Player iteration, elimination handling

### S-23: Simultaneous triggers from SBA deaths
- **Cards**: Multiple creatures dying simultaneously to SBA (e.g., Wrath of God resolves,
  3 creatures die), each with death triggers
- **Expected**: All SBA deaths happen simultaneously. All death triggers go on stack in
  APNAP order. Triggers from the same controller are ordered by that controller.
- **CR**: 704.3 (simultaneous SBA), 603.3b (APNAP)
- **Subsystems**: SBA batch processing, trigger ordering

---

## Priority 7: Niche But Known Engine-Breakers

### S-24: Panharmonicon + copy effect
- **Cards**: Panharmonicon, any creature with ETB, Spark Double (enters as copy)
- **Expected**: Spark Double copies a creature and enters. Panharmonicon sees a creature
  entering and doubles the ETB trigger. But Spark Double's own ETB (choose what to copy)
  is a self-ETB, not doubled by Panharmonicon. The copied creature's ETB IS doubled.
- **CR**: 603.2d (Panharmonicon filter), 707.10 (copy)
- **Subsystems**: Trigger doubling, copy effects, ETB trigger classification

### S-25: Morph face-down + ability removal
- **Cards**: Humility, any face-down creature (Morph)
- **Expected**: Face-down creature is already 2/2 with no abilities (CR 708.2). Humility
  also makes it 1/1 with no abilities. Layer ordering: face-down override is pre-layer,
  Humility applies at Layer 6 (abilities) and Layer 7b (P/T). Result: 1/1 no abilities.
  Turning face up is a special action, not an ability — still works under Humility.
- **CR**: 708.2, 702.37e (turning face up is special action)
- **Subsystems**: Face-down handling, layer system, special actions

### S-26: Mutate + zone change splitting
- **Cards**: Mutated creature (2+ cards), any sacrifice effect
- **Expected**: When mutated creature dies, all component cards go to graveyard as separate
  objects (CR 729.5). Each component's "when this dies" trigger fires separately.
- **CR**: 729.5 (zone-change splitting)
- **Subsystems**: Mutate zone splitting, trigger generation per component

### S-27: Day/Night + Blood Moon
- **Cards**: Any Daybound creature, Blood Moon
- **Expected**: Blood Moon doesn't affect creatures (only lands). Daybound/Nightbound
  transform triggers still function. The creature transforms based on day/night status.
- **CR**: 702.146, 305.7
- **Subsystems**: Day/night tracking, transform triggers, Blood Moon scope

---

## Conversion to Game Scripts

Each scenario should be converted to a JSON game script for the replay harness. The
script should:

1. Set up the initial board state with the exact cards needed
2. Execute the interaction step by step
3. Assert the correct game state at each critical point
4. Cite the CR rules being tested in the script metadata

Scripts go in `test-data/generated-scripts/stress-tests/` (new directory).

Priority 1 scenarios (S-01 through S-05) should be written alongside the layer bypass
fixes in M10. The rest can be written incrementally as capacity allows.
