# W-MISS Review — Batch 2 (12 cards)

**Reviewed**: 2026-07-17
**Cards**: 12
**Findings**: 1 HIGH, 0 MEDIUM, 0 LOW
**Verdict**: 11 correct & Complete; **1 must be demoted (Ojutai)**.

---

## Card 1: Goblin Wardriver — CORRECT
- Oracle/types/mana/P-T match ({R}{R}, Creature — Goblin Warrior, 2/2).
- Battle Cry modeled as `KeywordAbility::BattleCry` (SR-5 handled keyword). Correct. Complete OK.

## Card 2: Ojutai, Soul of Winter — HIGH (DEMOTE)
- Oracle/types/mana/P-T all match ({5}{W}{U}, Legendary Creature — Dragon, 5/6; Flying + Vigilance present).
- Dragon-attack trigger condition and filter are correct (`WheneverCreatureYouControlAttacks { filter: Dragon }` is dispatched and the subtype filter IS applied — `abilities.rs:6138`). `Effect::TapPermanent`, `Effect::PreventNextUntap`, and the opponent/non-land target filter are all correctly written, and `PreventNextUntap` is multiplayer-correct (sets `skip_untap_steps` on the target object, consumed at *its controller's* untap step).
- **F1 (HIGH): the trigger's TARGET is silently dropped, making the entire ability a no-op.**
  - Oracle clause: "tap target nonland permanent an opponent controls."
  - Root cause: `enrich_spec_from_def` converts this trigger to a runtime `TriggeredAbilityDef` and **hardcodes `targets: vec![]`** (`crates/engine/src/testing/replay_harness.rs:3011`) — the DSL's `targets: vec![TargetRequirement::TargetPermanentWithFilter(..)]` is not forwarded.
  - The stack-target fallback (`abilities.rs:6699-6713`) then tries the card registry, but it indexes `def.abilities.get(trigger.ability_index)`, and `trigger.ability_index` is the index into the runtime `triggered_abilities` vec (`abilities.rs:6090/6329`, idx 0), whereas `def.abilities[0]` is the `Flying` keyword — so the match arm `AbilityDefinition::Triggered { targets, .. }` fails and returns `vec![]`.
  - Net effect: `ability_targets` is empty, the trigger goes on the stack with no target, and `Effect::TapPermanent { target: DeclaredTarget { index: 0 } }` / `PreventNextUntap` resolve against an empty target list = **nothing is tapped, nothing is prevented**. Wrong game state (silent no-op), not merely omitted text.
  - No existing shipped card uses a *targeted* `WheneverCreatureYouControlAttacks` (kolaghan, dromoka, utvara, kazuul, etc. all pass `targets: vec![]`), so this path has never worked. Ojutai is the first to exercise it.
  - **Fix**: this is a genuine engine/DSL gap. Ojutai cannot be `completeness: Complete`. Either (a) demote to `KnownWrong` with note "targeted WheneverCreatureYouControlAttacks trigger unsupported — enrich hardcodes targets: vec![] and the registry fallback mis-indexes ability_index (triggered_abilities idx vs def.abilities idx)"; or (b) fix the primitive: forward the DSL `targets` into the runtime `TriggeredAbilityDef` in the enrich block, and/or make the fallback resolve the Triggered ability by matching among `def.abilities` triggered entries rather than by raw index. Keep the current abilities as-is once the primitive lands.

## Card 3: Rhys the Exiled — CORRECT
- Oracle/types/mana/P-T match ({2}{G}, Legendary Creature — Elf Warrior, 3/2).
- `WhenAttacks` → `GainLife` with `EffectAmount::PermanentCount { filter: Elf, controller: Controller }`. Correct scope: "you gain 1 life for each Elf **you control**" → Controller is right (NOT EachPlayer/battlefield). Variant is real (`effects/mod.rs:6749`).
- `{B}, Sacrifice an Elf: Regenerate Rhys` → `Cost::Sequence([Mana{B}, Sacrifice(Elf)])`, `Effect::Regenerate(Source)`. Correct. Complete OK.

## Card 4: Triumphant Adventurer — CORRECT
- Oracle/types/mana match ({W}{B}, Creature — Human Knight); P/T 1/1 is the paper printing (the 2/1 is the A- Alchemy face — correctly not used).
- Deathtouch keyword; venture-on-attack (`Effect::VentureIntoDungeon`, real at `effects/mod.rs:3755`).
- "During your turn, this creature has first strike" → `Static` continuous effect (Layer 6 AddKeyword FirstStrike) gated on `Condition::IsYourTurn`. Confirmed the layer system evaluates `ContinuousEffectDef.condition` and skips application when false (`layers.rs:520-529`; `IsYourTurn` = `turn.active_player == controller`). Correctly does NOT grant first strike on opponents' turns. Complete OK.

## Card 5: Aggravated Assault — CORRECT
- Oracle/type/mana match ({2}{R}, Enchantment, no P/T).
- `{3}{R}{R}` activated, `TimingRestriction::SorcerySpeed` (correct for "Activate only as a sorcery"), `UntapAll` creatures you control + `AdditionalCombatPhase { followed_by_main: true }` (real, `effects/mod.rs:2385`; pushes PostCombatMain then Combat, guarded on active_player == controller which holds under sorcery timing). Complete OK.

## Card 6: Hyrax Tower Scout — CORRECT
- {2}{G}, Creature — Human Scout, 3/3; ETB `UntapPermanent` targeting `TargetCreature`. Matches oracle. Complete OK.

## Card 7: Mobilize — CORRECT
- {G}, Sorcery (the sorcery "Untap all creatures you control", not the Olivia/keyword faces). `Spell` → `UntapAll { Creature, You }`. Complete OK.

## Card 8: Vitalize — CORRECT
- {G}, Instant, "Untap all creatures you control". Correct card type (Instant, distinct from Mobilize's Sorcery). Complete OK.

## Card 9: Wilderness Reclamation — CORRECT
- {3}{G}, Enchantment; `AtBeginningOfYourEndStep` → `UntapAll { Land, You }`. Trigger dispatched for active player's permanents (`turn_actions.rs:677-711`). Complete OK.

## Card 10: Wheel of Fortune — CORRECT
- {2}{R}, Sorcery; `WheelHand { EachPlayer, Discard, Fixed(7) }`. Correct player scope (each player) and count. Complete OK.

## Card 11: Tolarian Winds — CORRECT
- {1}{U}, Instant; `WheelHand { Controller, Discard, ThatMany }`. Correct — self only, draw = discarded count (hand size snapshot before discard, `effects/mod.rs:617-639`). Complete OK.

## Card 12: Fateful Showdown — CORRECT
- {2}{R}{R}, Instant. Sequence: `DealDamage { DeclaredTarget 0, HandSize(Controller) }` then `WheelHand { Controller, Discard, ThatMany }`; target `TargetAny`. Damage = your hand size (locked at execution, hand still full), then discard hand + draw that many. Amount, target, and ordering all correct. `EffectAmount::HandSize` is real (`effects/mod.rs:7077`). Complete OK.

---

## Summary
- **Needs fix / demotion**: Ojutai, Soul of Winter (HIGH — targeted attack-trigger is a silent no-op; not authorable as Complete until the enrich/target-forwarding primitive is fixed).
- **Clean & Complete (11)**: Goblin Wardriver, Rhys the Exiled, Triumphant Adventurer, Aggravated Assault, Hyrax Tower Scout, Mobilize, Vitalize, Wilderness Reclamation, Wheel of Fortune, Tolarian Winds, Fateful Showdown.
- No gated stub effects (`Choose`/`MayPayOrElse`/`AddManaChoice`/`AddManaAnyColor*`) used anywhere in the batch.
