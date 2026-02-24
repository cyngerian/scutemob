# MTG Rules Engine: Corner Case Interaction Test Reference

> This document catalogs known difficult interactions that any MTG rules engine must handle
> correctly. Each entry describes the interaction, what rules subsystem it stress-tests,
> the expected correct behavior, and which CR sections govern it.
>
> This is a living document — add new cases as they're discovered during development.

---

## Layer System Interactions

These test the continuous effect layer system (CR 613), dependency resolution, and timestamp ordering. The layer system is the single hardest subsystem to implement correctly.

### 1. Humility + Opalescence

**Cards**: Humility (enchantment — "All creatures lose all abilities and have base power and toughness 1/1"), Opalescence (enchantment — "Each other non-Aura enchantment is a creature in addition to its other types and has base power and toughness each equal to its mana value")

**What's being tested**: Layer interaction between type-changing (layer 4), ability-removing (layer 6), and P/T-setting (layer 7b) effects. Dependency detection — Humility's effect depends on Opalescence's because Opalescence determines whether Humility is a creature.

**Correct behavior**: Both are creatures (Opalescence makes enchantments into creatures in layer 4, which applies before Humility's effects in layers 6 and 7). Both lose all abilities (layer 6). Both are 1/1 (Humility's P/T-setting in layer 7b applies after Opalescence's, regardless of timestamp, because of the dependency). The critical subtlety: even though both lose abilities, the effects are granted by the enchantments *as continuous effects* — losing the abilities on the permanent doesn't remove the continuous effects already in play.

**CR sections**: 613.1 (layer ordering), 613.8 (dependency), 613.10 (specific example in the CR)

---

### 2. Blood Moon + Urborg, Tomb of Yawgmoth

**Cards**: Blood Moon (enchantment — "Nonbasic lands are Mountains"), Urborg, Tomb of Yawgmoth (legendary land — "Each land is a Swamp in addition to its other types")

**What's being tested**: Dependency within layer 4 (type-changing effects). Blood Moon removes Urborg's ability by making it a Mountain (losing all other abilities). But Urborg's effect makes lands into Swamps. The order matters, and dependency determines it.

**Correct behavior**: Blood Moon's effect depends on Urborg's (because Urborg's effect could change what Blood Moon's effect does — if Urborg makes Blood Moon's source a Swamp, that's relevant). Wait — actually Blood Moon makes nonbasic lands into Mountains, which removes their abilities in layer 4. Since Urborg is nonbasic, Blood Moon makes it a Mountain, stripping its ability. Urborg's effect never applies. All nonbasic lands are Mountains (not Swamp Mountains). Basic lands are unaffected by Blood Moon and also are not given the Swamp type (Urborg's ability was removed).

**CR sections**: 613.1d (layer 4), 613.8 (dependency), 305.7 (subtypes of lands)

---

### 3. Blood Moon + Urborg (Reversed Timestamp)

**Cards**: Same as above, but Urborg entered the battlefield AFTER Blood Moon.

**What's being tested**: Whether timestamp matters when dependency exists. It shouldn't — dependency overrides timestamp per CR 613.8.

**Correct behavior**: Same as #2 regardless of timestamp. Blood Moon still wins because the dependency relationship is the same. This specifically tests that your engine doesn't fall back to timestamp when a real dependency exists.

**CR sections**: 613.8a-b (dependency overrides timestamp)

---

### 4. Yixlid Jailer + Anger (in graveyard)

**Cards**: Yixlid Jailer (creature — "Cards in graveyards lose all abilities"), Anger (creature — "As long as Anger is in your graveyard and you control a Mountain, creatures you control have haste")

**What's being tested**: Layer 6 (ability-removing/adding) with a dependency. Anger's ability is a static ability that functions from the graveyard. Yixlid Jailer removes that ability. But does Anger's ability even exist to be removed?

**Correct behavior**: Yixlid Jailer's ability applies in layer 6. Anger's ability is removed. Your creatures do NOT have haste. The dependency here is straightforward — Jailer's effect removes the very ability that would grant haste.

**CR sections**: 613.1f (layer 6), 112.6 (abilities that function in non-battlefield zones)

---

### 5. Multiple Copy Effects (Clone copying a Clone)

**Cards**: Clone (creature — "As Clone enters the battlefield, you may choose a creature on the battlefield. If you do, Clone enters the battlefield as a copy of that creature"), copying another Clone that already copied something.

**What's being tested**: Layer 1 (copy effects), copiable values (CR 707.2). When Clone copies a Clone-that-copied-a-Bear, what do you get? Copy effects form a chain in layer 1.

**Correct behavior**: You get a copy of whatever the first Clone copied (the Bear), not a copy of Clone itself. Copiable values of an object include modifications from other copy effects. The second Clone sees the first Clone's copiable values, which are already "Bear."

**CR sections**: 707.2 (copiable values), 707.3 (copy effects copy the modified copiable values)

---

### 6. Humility + Magus of the Moon (Timestamp Dependent)

**Cards**: Humility (see above), Magus of the Moon (creature — "Nonbasic lands are Mountains")

**What's being tested**: Dependency analysis when two effects could each affect the other, creating potential circularity. Humility removes Magus's ability. But Magus's ability changes land types. These are in different layers (6 vs 4), so they're actually not dependent — layer 4 applies before layer 6.

**Correct behavior**: Layer 4: Magus's ability makes nonbasic lands Mountains (this happens first). Layer 6: Humility removes all creature abilities (including Magus's, but it already applied in layer 4). Layer 7b: all creatures are 1/1. So nonbasic lands ARE Mountains, AND all creatures are 1/1 with no abilities.

**CR sections**: 613.1 (layer ordering makes this non-dependent even though it feels like it should be)

---

### 7. Opalescence + Parallax Wave (Timestamp Matters)

**Cards**: Opalescence (see above), Parallax Wave (enchantment with 5 fade counters — activated ability to exile a creature)

**What's being tested**: Opalescence makes Parallax Wave a creature (P/T equal to its mana value, so 4/4). If Parallax Wave uses its ability to exile creatures, and then Parallax Wave leaves the battlefield, all exiled creatures return. But since Parallax Wave is a creature (via Opalescence), it can be killed, triggering the return of exiled cards. This tests the interaction of continuous effects with zone changes.

**Correct behavior**: When Parallax Wave leaves the battlefield, all cards exiled with it return to the battlefield. This works normally — the key test is that the engine handles Opalescence making Parallax Wave a valid target for creature removal, and that zone-change triggers from the exiled cards fire correctly.

**CR sections**: 613.1d (type-changing layer), 400.7 (zone-change object identity)

---

## State-Based Action Interactions

### 8. Deathtouch + Trample (Damage Assignment)

**Cards**: Any creature with both deathtouch and trample attacking, blocked by a creature.

**What's being tested**: The interaction between deathtouch's "any amount of damage is lethal" (CR 704.5h) and trample's "assign lethal damage to blocker, rest to defending player" (CR 702.19c). The combination of these rules means only 1 damage needs to be assigned to the blocker.

**Correct behavior**: A 5/5 with deathtouch and trample blocked by a 10/10 assigns 1 damage to the blocker (lethal due to deathtouch) and 4 damage to the defending player. The blocker is destroyed via SBA (deathtouch damage).

**CR sections**: 702.2 (deathtouch), 702.19c (trample), 704.5h (SBA — creature dealt deathtouch damage is destroyed)

---

### 9. Indestructible + Deathtouch

**Cards**: Indestructible creature blocking a creature with deathtouch.

**What's being tested**: Deathtouch causes destruction via SBA 704.5h. Indestructible prevents destruction. These interact correctly — the creature is dealt damage (and has been dealt damage by a deathtouch source), but the SBA destruction is prevented by indestructible.

**Correct behavior**: The indestructible creature survives. It has taken damage (which may matter for other effects) but is not destroyed. Note: if the damage reduces toughness to 0 (e.g., via -X/-X effects, NOT damage), indestructible doesn't help — SBA 704.5f (0 toughness) is not destruction.

**CR sections**: 702.2 (deathtouch), 702.12 (indestructible), 704.5f vs 704.5h (different SBAs)

---

### 10. Legendary Rule with Mutiple Copies Simultaneously

**Cards**: Two copies of the same legendary permanent entering the battlefield simultaneously (e.g., from a mass reanimation effect).

**What's being tested**: SBA 704.5j — legendary rule. When a player controls two or more legendary permanents with the same name, they choose one to keep and put the rest into the graveyard. When both enter simultaneously, neither was "already there" — both are subject to the legend rule at the same time.

**Correct behavior**: Both enter the battlefield. SBAs are checked. The legend rule triggers. The controller chooses one to keep and puts the other into the graveyard. This matters because both briefly existed on the battlefield — triggers that fire on ETB fire for BOTH, even though one immediately goes to the graveyard.

**CR sections**: 704.5j (legend rule), 603.6a (triggers from events that happen simultaneously)

---

### 11. +1/+1 and -1/-1 Counter Annihilation

**Cards**: Any creature with both +1/+1 and -1/-1 counters on it.

**What's being tested**: SBA 704.5q — these counters annihilate in pairs. A creature with 3 +1/+1 counters and 2 -1/-1 counters ends up with 1 +1/+1 counter.

**Correct behavior**: During SBA check, remove matching pairs until only one type remains. This is simultaneous — it's one SBA application, not iterative. This matters because some cards care about counters being removed.

**CR sections**: 704.5q (counter annihilation), 122.3 (counter rules)

---

## Stack and Priority Interactions

### 12. Spell Fizzle (All Targets Illegal on Resolution)

**Cards**: Any targeted spell where all targets become illegal between casting and resolution (e.g., target creature is sacrificed in response).

**What's being tested**: CR 608.2b — a spell that has targets is countered by the game rules if all its targets are illegal on resolution.

**Correct behavior**: The spell is countered (put into its owner's graveyard, or exile if exiled). None of its effects happen — even effects that don't target. This is critical: a spell with "destroy target creature, then draw a card" does NOTHING if the creature is gone.

**CR sections**: 608.2b (fizzle rule)

---

### 13. Partial Fizzle (Some Targets Illegal)

**Cards**: A spell with multiple targets where some but not all become illegal.

**What's being tested**: CR 608.2b — if a spell has multiple targets and at least one is still legal, the spell resolves. It does as much as it can, ignoring illegal targets.

**Correct behavior**: The spell resolves. Effects that reference the illegal target don't happen for that target. Effects that reference legal targets happen normally. Effects that don't reference any specific target happen normally.

**CR sections**: 608.2b (partial resolution)

---

### 14. APNAP Trigger Ordering in Multiplayer

**Cards**: An effect that triggers abilities controlled by multiple players simultaneously (e.g., Wrath of God destroying all creatures in a 4-player game).

**What's being tested**: CR 603.3b — when multiple triggered abilities trigger simultaneously, they're put on the stack in APNAP (Active Player, Non-Active Player) order. Each player orders their own triggers, then the next player in turn order does the same.

**Correct behavior**: Active player's triggers go on the stack first (and thus resolve LAST). Then the next player in turn order, and so on. Within each player's triggers, that player chooses the order. This means the last player in turn order has their triggers resolve first — a significant strategic consideration in Commander.

**CR sections**: 603.3b (APNAP ordering), 101.4 (turn order in multiplayer)

---

### 15. Panharmonicon-Style Trigger Doubling

**Cards**: Panharmonicon (artifact — "If an artifact or creature entering the battlefield causes a triggered ability of a permanent you control to trigger, that ability triggers an additional time")

**What's being tested**: The engine's ability to handle trigger-modifying effects. Panharmonicon doesn't add a copy of the trigger to the stack — it makes the event trigger the ability twice. Both triggers are real triggered abilities that go on the stack and can be responded to individually.

**Correct behavior**: When a creature ETBs, each triggered ability it triggers goes on the stack twice. The controller orders all of these triggers. Each resolves independently. Removing Panharmonicon in response to the first trigger doesn't prevent the second — the trigger already triggered.

**CR sections**: 603.2 (trigger events), Panharmonicon's specific oracle text creates a replacement-like modification of the trigger event

---

## Replacement Effect Interactions

### 16. Multiple Replacement Effects (Player Chooses Order)

**Cards**: Two replacement effects that could both apply to the same event (e.g., Rest in Peace + Leyline of the Void when a creature an opponent controls dies).

**What's being tested**: CR 616.1 — when multiple replacement effects could apply to the same event, the affected player or controller of the affected object chooses which applies first. After one applies, the others are checked again (the modified event might no longer match).

**Correct behavior**: The affected player (or controller) chooses which replacement effect to apply first. After it applies, check if the remaining effects still apply to the (now modified) event. This is iterative — each application could change whether other effects still apply.

**CR sections**: 614.5 (replacement effect applies once), 616.1 (ordering multiple replacements)

---

### 17. Self-Replacement Effects Apply First

**Cards**: A card with a self-replacement effect and an external replacement effect both applicable. Example: a creature with "If ~ would die, exile it instead" and Rest in Peace ("If a nontoken creature would be put into a graveyard from anywhere, exile it instead").

**What's being tested**: CR 614.15 — self-replacement effects (effects that modify how the card itself does something) apply before other replacement effects, regardless of player choice.

**Correct behavior**: The self-replacement effect applies first. If it changes the event enough that the external replacement effect no longer applies, the external one is skipped.

**CR sections**: 614.15 (self-replacement priority)

---

### 18. Commander Zone-Change Replacement + Rest in Peace

**Cards**: A commander that would die while Rest in Peace is on the battlefield.

**What's being tested**: Two replacement effects both want to modify the "goes to graveyard" event. Rest in Peace replaces "to graveyard" with "to exile." The commander replacement effect replaces "to graveyard or exile" with "to command zone" (if the owner chooses). The order matters and the controller chooses.

**Correct behavior**: The commander's controller chooses which replacement to apply first. If they choose the commander replacement, the commander goes to the command zone (Rest in Peace never applies because the event is no longer "goes to graveyard"). If they choose Rest in Peace first, the event becomes "goes to exile," and then the commander replacement can still apply (it also covers exile), sending it to the command zone.

Either way, the commander can end up in the command zone — but the controller must choose correctly.

**CR sections**: 903.9 (commander zone change), 614.5, 616.1

---

### 19. "Enters Tapped" Replacement Effects

**Cards**: A land with "enters the battlefield tapped" and an effect that says "permanents enter the battlefield with an additional +1/+1 counter" (e.g., Vorinclex, Monstrous Raider doubling counters).

**What's being tested**: ETB replacement effects modify how the permanent enters — they're not triggered abilities. They apply before the permanent is on the battlefield. The permanent was never "untapped on the battlefield" — it arrived tapped.

**Correct behavior**: The land enters tapped. This is not a triggered ability and cannot be responded to. Effects that look for "whenever a land enters the battlefield untapped" do NOT trigger.

**CR sections**: 614.1c (enters-the-battlefield replacement effects)

---

## Combat Interactions

### 20. First Strike + Double Strike Ordering

**Cards**: A creature with first strike blocking a creature with double strike.

**What's being tested**: The combat damage step splits into two when any creature has first strike or double strike. First/double strikers deal damage in the first combat damage step. Non-first-strikers deal damage in the second. Double strikers deal damage in BOTH.

**Correct behavior**: First combat damage step: both the first striker and double striker deal damage (first striker at its power, double striker at its power). If the first striker dies from the double striker's damage, it doesn't deal damage in the second step (it's already dead). Second combat damage step: only the double striker deals damage again (the first striker already dealt its damage in the first step and doesn't deal again — it has first strike, not double strike).

**CR sections**: 702.4 (first strike), 702.4b (double strike), 510 (combat damage step)

---

### 21. Protection from X (All Four Effects)

**Cards**: A creature with "Protection from red" being targeted by a red spell, blocked by a red creature, enchanted by a red aura, and dealt damage by a red source.

**What's being tested**: The DEBT acronym — protection prevents **D**amage, **E**nchanting/**E**quipping, **B**locking, and **T**argeting from sources with the quality. Each of these is a separate rules interaction.

**Correct behavior**: 
- Damage from red sources is prevented (even non-targeted damage like Earthquake)
- Red auras cannot be attached (existing ones fall off via SBA)
- Red creatures cannot block it
- Red spells/abilities cannot target it
- HOWEVER: non-targeted red effects that don't deal damage still work (e.g., "all creatures get -1/-1" from a red source — this works because it doesn't target and -1/-1 is not damage)

**CR sections**: 702.16 (protection)

---

### 22. Hexproof vs. "Each Opponent" Effects

**Cards**: A hexproof creature and a spell that says "each creature gets -1/-1 until end of turn."

**What's being tested**: Hexproof prevents targeting only. Effects that don't use the word "target" affect hexproof creatures normally. Many newer players (and some engines) get this wrong.

**Correct behavior**: The hexproof creature gets -1/-1. The spell doesn't target — it affects all creatures. Hexproof is irrelevant. Similarly, board wipes like Wrath of God destroy hexproof creatures because they don't target.

**CR sections**: 702.11 (hexproof — "can't be the target of spells or abilities your opponents control")

---

## Zone Change and Object Identity

### 23. "When This Creature Dies" + Flicker

**Cards**: A creature with "When this creature dies, draw a card" that is exiled and immediately returned to the battlefield (flickered) in response to a kill spell.

**What's being tested**: Object identity (CR 400.7). When the creature is exiled and returns, it's a new object. The kill spell fizzles (target is gone). But what about if the creature is killed normally — the "dies" trigger must reference the object that actually died, not the new one on the battlefield.

**Correct behavior**: The kill spell fizzles (creature is no longer on the battlefield — it left and a new object returned). No dies trigger fires (the creature wasn't destroyed — it was exiled, then a new object entered). The new object on the battlefield has no memory of being targeted.

**CR sections**: 400.7 (zone change creates new object), 608.2b (fizzle)

---

### 24. Tokens Briefly Existing in Non-Battlefield Zones

**Cards**: A token creature that dies with a "whenever a creature dies" trigger on the battlefield.

**What's being tested**: CR 111.8 — tokens that leave the battlefield briefly exist in the new zone before ceasing to exist (via SBA 704.5d). This means "dies" triggers DO fire for tokens.

**Correct behavior**: The token moves to the graveyard. "Whenever a creature dies" triggers fire (the token was in the graveyard long enough for the game to see it). Then SBAs are checked and the token ceases to exist in the graveyard. It cannot be returned to the battlefield (it's gone), but the trigger already fired.

**CR sections**: 111.8 (tokens in non-battlefield zones), 704.5d (token SBA)

---

### 25. Phasing and Auras/Equipment

**Cards**: An equipped/enchanted creature that phases out.

**What's being tested**: CR 702.26d — when a permanent phases out, auras and equipment attached to it phase out WITH it (they're "indirectly phased out"). When it phases back in, they phase in still attached. This is different from leaving the battlefield — auras don't fall off, equipment doesn't detach.

**Correct behavior**: Creature phases out. Aura and equipment phase out with it (indirectly phased). All three are treated as though they don't exist. On the controller's next untap step, all three phase in together, still attached. No ETB triggers fire (phasing in is not entering the battlefield).

**CR sections**: 702.26d (indirect phasing), 702.26e (phasing in is not ETB)

---

## Commander-Specific Interactions

### 26. Commander Damage from a Copy

**Cards**: A copy of an opponent's commander dealing combat damage to you (e.g., via Clone).

**What's being tested**: Commander damage (CR 903.10) is tracked per commander, not per creature. A copy of a commander is NOT a commander (unless it's somehow in the command zone, which copies can't be).

**Correct behavior**: The copy deals combat damage normally, but it does NOT count as commander damage. The copy is not a commander — it's a creature that happens to have the same characteristics. Commander damage is a property of the physical card designated as a commander at game start.

**CR sections**: 903.3 (commander designation), 903.10 (commander damage), 707.2 (copies don't copy external designations)

---

### 27. Commander Tax with Partner Commanders

**Cards**: Two partner commanders, each cast multiple times from the command zone.

**What's being tested**: Commander tax is tracked per-commander. Casting Commander A three times adds {6} to A's cost but doesn't affect Commander B's cost.

**Correct behavior**: Commander A costs {2} extra per previous cast from command zone. Commander B tracks its own count independently. If A has been cast 3 times and B has been cast once, A costs +{6} and B costs +{2}.

**CR sections**: 903.8 (commander tax), 702.124 (partner)

---

### 28. Commander Dies with Replacement Effect That Exiles

**Cards**: Your commander is destroyed while Kalitas, Traitor of Ghet is on the battlefield under an opponent's control ("If a nontoken creature an opponent controls would die, instead exile that creature and create a 2/2 Zombie token").

**What's being tested**: Kalitas's ability is a replacement effect that changes "dies" (goes to graveyard) to "exile." The commander replacement effect can also apply. The commander's owner chooses the order.

**Correct behavior**: Two replacement effects apply to "commander would go to graveyard." The commander's owner chooses order:
- If they apply Kalitas first: commander is exiled instead. Then the commander replacement can apply again (it covers exile too) — commander goes to command zone. Kalitas's controller gets a Zombie token (the exile happened, even though it was further replaced).
- If they apply commander replacement first: commander goes to command zone instead of graveyard. Kalitas's replacement no longer applies (the event is no longer "would die"). No Zombie token.

The owner should choose to apply the commander replacement first to avoid giving the opponent a Zombie.

**CR sections**: 903.9, 616.1 (ordering replacements)

---

## Miscellaneous Complex Interactions

### 29. Cascade into a Split Card

**Cards**: A card with cascade (e.g., Bloodbraid Elf, MV 4) cascading into a split card (e.g., Wear // Tear, MV 3 // 1).

**What's being tested**: How mana value is calculated for split cards in different zones. On the stack, a split card has the MV of the half being cast. In every other zone, it has the combined MV of both halves.

**Correct behavior**: While in the library during cascade, Wear // Tear has a combined MV of 4 (3+1). Cascade from a 4-MV spell exiles cards until it finds one with MV less than 4. Wear // Tear has MV 4, so cascade skips it. This catches engines that incorrectly check each half separately.

**CR sections**: 708.4 (split card MV), 702.84 (cascade)

---

### 30. Morph/Manifest Face-Down Characteristics

**Cards**: A face-down creature (via morph or manifest).

**What's being tested**: CR 707.2 and 708.4 — a face-down creature has no name, no mana cost, is a 2/2 creature with no abilities, no color, no types beyond creature. Its actual characteristics are hidden. Continuous effects that check characteristics see only the face-down values.

**Correct behavior**: Face-down creature is a 2/2 colorless creature with no name, no abilities, no creature types. Humility still makes it 1/1 (it's already a creature). "Destroy all Humans" doesn't kill it (it has no creature types). It can be turned face up by paying its morph cost (this is a special action that doesn't use the stack).

**CR sections**: 708 (face-down spells and permanents), 707.2 (copiable values of face-down)

---

### 31. Aura Attached to Illegal Permanent After Type Change

**Cards**: An Aura that says "Enchant creature" attached to an animated land (a land that's also a creature). The animation effect ends.

**What's being tested**: SBA 704.5m — if an Aura is attached to an illegal object, it's put into its owner's graveyard. When the land stops being a creature, "Enchant creature" is no longer satisfied.

**Correct behavior**: As soon as the animation effect ends (e.g., at end of turn), the land is no longer a creature. On the next SBA check, the Aura is no longer legally attached. The Aura goes to the graveyard.

**CR sections**: 704.5m (illegal Aura SBA), 303.4c (Aura legality)

---

### 32. Mutate Stack Ordering

**Cards**: Multiple mutate creatures on a single permanent.

**What's being tested**: Mutate creates a "merged permanent" — an ordered pile of cards. The topmost card determines the merged permanent's characteristics. Abilities from all cards in the pile apply. The order matters and the controller chooses.

**Correct behavior**: When a creature mutates onto another, the controller chooses over or under. The topmost creature card determines name, mana cost, P/T, types, art (for display). All abilities from all cards in the merged pile apply. If the merged permanent is bounced, all cards go to hand. If it dies, all cards go to graveyard.

**CR sections**: 725 (merging with permanents)

---

### 33. Sylvan Library: Drawing and Replacement Effects

**Cards**: Sylvan Library ("At the beginning of your draw step, you may draw two additional cards. If you do, choose two cards in your hand drawn this turn. For each of those cards, pay 4 life or put the card on top of your library.")

**What's being tested**: Interaction with effects that replace draws (e.g., Dredge) or modify them (e.g., Underrealm Lich). If a draw is replaced, Sylvan Library can't track "cards drawn this turn" for that replaced draw, because no card was drawn.

**Correct behavior**: If you replace one of the Sylvan Library draws with Dredge, that wasn't a "draw" — you can't be forced to put a card back for it. This reduces the number of cards you must account for. This interaction is notorious for confusing judges.

**CR sections**: 614 (replacement effects modifying draws), Sylvan Library's specific Oracle rulings

---

### 34. Simultaneously Entering Effects (Reveillark + Karmic Guide Loop)

**Cards**: Reveillark ("When Reveillark leaves the battlefield, return up to two target creatures with power 2 or less from your graveyard to the battlefield") + Karmic Guide ("When Karmic Guide enters the battlefield, return target creature card from your graveyard to the battlefield") + a sacrifice outlet.

**What's being tested**: This is a classic infinite loop enabled by the stack. Sacrifice Reveillark → triggers return Karmic Guide → Karmic Guide ETB triggers returns Reveillark → sacrifice Reveillark again. The engine must detect and handle mandatory infinite loops per CR 726.

**Correct behavior**: The loop continues until a player breaks it or a game state change makes it stop. If the loop is all mandatory triggers with no optional components, the game is a draw unless a player can interrupt it. The engine must detect the loop (same game state recurring) and prompt for intervention or declare a draw.

**CR sections**: 726 (handling infinite loops), 104.4b (infinite mandatory loop = draw)

---

### 35. Chains of Triggered Abilities (Storm + Copying)

**Cards**: A spell with Storm ("When you cast this spell, copy it for each spell cast before it this turn") targeting various things.

**What's being tested**: The copies are put directly on the stack above the original. Each copy is a separate stack object with its own targets. Resolving 10+ stack objects in sequence, each potentially triggering other abilities (e.g., Guttersnipe — "Whenever you cast or copy an instant or sorcery spell, Guttersnipe deals 2 damage to each opponent").

**Correct behavior**: Storm creates copies on cast (they go on top of the stack). Each copy can have new targets chosen. Copies resolve one at a time, with priority between each resolution. Any triggered abilities from copies resolving go on the stack between resolutions.

**CR sections**: 702.40 (storm), 707.10 (copies of spells)

---

### 36. Blood Moon + Urza's Saga (Layer 4 vs Layer 6, Saga Sacrifice SBA)

**Cards**: Blood Moon (enchantment — "Nonbasic lands are Mountains"), Urza's Saga (Enchantment Land — Urza's Saga, with three chapter abilities that grant abilities to the Saga when they resolve)

**What's being tested**: Two distinct interactions triggered by the June 2025 CR 714.4 update:
1. **Saga sacrifice SBA**: CR 714.4 now reads "a Saga permanent *with one or more chapter abilities*" — when Blood Moon removes all chapter abilities, the SBA condition is never met and the Saga is NOT sacrificed.
2. **Layer 4 vs Layer 6 for gained abilities**: Blood Moon's type-change applies in Layer 4, which strips Urza's Saga's printed chapter abilities. However, abilities *gained* as a result of previously resolved chapter abilities (e.g., `{T}: Add {C}` from Chapter I, the Construct-making ability from Chapter II) are separate continuous effects applied in Layer 6, which comes *after* Layer 4. Therefore those gained abilities survive Blood Moon.

**Correct behavior** (post-June 2025 CR):
- Urza's Saga is NOT sacrificed under Blood Moon, regardless of how many lore counters it has.
- Urza's Saga retains any abilities gained from chapter abilities that already resolved (because those are Layer 6 effects with timestamps later than Blood Moon's Layer 4 effect, if Blood Moon entered before those chapters resolved; or with earlier timestamps if Blood Moon entered after — in that case the gained abilities have *earlier* timestamps and Blood Moon's Layer 6 removal overrides them — so the exact behavior depends on entry order).
- No further lore counters are added while Blood Moon is in effect, so no new chapter abilities trigger.
- Alpine Moon (which explicitly says "lose all abilities" in its text) applies in Layer 6 and will override gained abilities, unlike Blood Moon.

**Entry order matters for retained abilities**:
- Blood Moon entered *before* Urza's Saga chapters resolved → chapter gains have *later* timestamps → gained abilities survive (Layer 6 timestamp ordering).
- Blood Moon entered *after* Urza's Saga chapters resolved → Blood Moon has *later* timestamp in Layer 6 → gained abilities are removed.

**CR sections**: 714.4 (Saga sacrifice SBA — updated June 2025), 613.1d (layer 4 type-changing), 613.1f (layer 6 ability-adding/removing), 613.7 (timestamp ordering within layers)

---

## Testing Priority by Rules Subsystem

| Subsystem | Test Cases | Priority |
|-----------|-----------|----------|
| Layer System (613) | 1-7, 30, 36 | Critical — implement and test in M5 |
| SBAs (704) | 8-11, 24, 31 | Critical — implement and test in M4 |
| Stack/Priority (405, 117) | 12-15, 35 | Critical — implement and test in M3 |
| Replacement Effects (614-616) | 16-19, 28, 33 | High — implement in M8 |
| Combat (500s, 702) | 8, 20-21, 22 | High — implement in M6 |
| Zone Changes (400.7) | 23-25, 31 | High — implement in M1 (model) and M3+ (behavior) |
| Commander-Specific (903) | 18, 26-28 | High — implement in M9 |
| Complex Interactions | 29, 32, 34, 35 | Medium — validate after engine core complete |
| Infinite Loop Detection (726) | 34 | Medium — needed before alpha |
