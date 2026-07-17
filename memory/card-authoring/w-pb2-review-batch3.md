# Card Review: W-PB2 Batch 3 — dynamic-P/T / static-grant defs

**Reviewed**: 2026-07-17
**Cards**: 12
**Findings**: 0 HIGH, 1 MEDIUM, 3 LOW

Engine claims verified against source (all three load-bearing):
- **meathook X-propagation**: `resolution.rs` L2099-2105 sets `ctx.x_value = state.objects.get(&source_object).map(|o| o.x_value)` for the ETB path; `obj.x_value = stack_obj.x_value` at L606 on enter. X threads into the ETB trigger context — `EffectAmount::XValue` reads the cast X, not 0. CONFIRMED.
- **elven_chorus any-color stub**: `rules/mana.rs` L337-338 and L354-365 — every `any_color: true` ManaAbility unconditionally adds `ManaColor::Colorless` ("Simplified: colorless until interactive color choice is implemented"). The grant would ship wrong game state. Author's finding REAL.
- **goblin modal-activated gap**: `card_definition.rs` L285 `Activated` variant has no `modes` field (Triggered L330 / Spell L354 do). CONFIRMED — modal activated ability can only route through gated `Effect::Choose`.
- **olivia duration gap**: `continuous_effect.rs` L44-64 `EffectDuration` has no "while you control source" variant. CONFIRMED — divergence is unfixable without a new primitive.

---

## Card 1: Aspect of Hydra
- **Oracle match**: NO (reminder text only)
- **Types match**: YES (Instant)
- **Mana cost match**: YES ({G})
- **DSL correctness**: YES
- **Findings**:
  - F1 (LOW, KI-18): Reminder text diverges from Scryfall. Def L15-18 reads "(Your devotion to green is the number of green mana symbols in the mana costs of permanents you control.)"; Scryfall reads "(Each {G} in the mana costs of permanents you control counts toward your devotion to green.)". Rules-neutral; `EffectAmount::DevotionTo(Color::Green)` on `ModifyBothDynamic` is correct.
- **Verdict**: PASS

## Card 2: Craterhoof Behemoth
- **Oracle match**: YES
- **Types match**: YES (Creature — Beast, 5/5)
- **Mana cost match**: YES ({5}{G}{G}{G})
- **DSL correctness**: YES
- **Findings**: none. ETB `ModifyBothDynamic{PermanentCount(creature, Controller)}` on `CreaturesYouControl` + `AddKeywords(Trample)`, both `UntilEndOfTurn`; Haste as keyword. X = creatures-you-control (includes self) locked at resolution. Matches oracle exactly.
- **Verdict**: PASS

## Card 3: Devilish Valet
- **Oracle match**: YES
- **Types match**: YES (Creature — Devil Warrior, 1/3)
- **Mana cost match**: YES ({2}{R})
- **DSL correctness**: YES
- **Findings**: none. Trigger `WheneverCreatureEntersBattlefield{controller: You, exclude_self: true}` — "another creature you control" correctly excludes self (def L35). "Double this creature's power" = `ModifyPowerDynamic{PowerOf(Source)}` on `Source`, `UntilEndOfTurn` — adds current power (locked at resolution), so repeated triggers compound correctly. Alliance is an ability word (flavor), correctly not modeled as a keyword.
- **Verdict**: PASS

## Card 4: The Meathook Massacre
- **Oracle match**: YES
- **Types match**: YES (Legendary Enchantment — supertype present, def L13)
- **Mana cost match**: YES ({X}{B}{B} → black:2, x_count:1)
- **DSL correctness**: YES
- **Findings**: none.
  - ETB `-X/-X` to `AllCreatures` via `ModifyBothDynamic{XValue, negate:true}` — X propagation confirmed in engine (see header). NOT the "unsubstituted XValue reads 0" case.
  - Death trigger 1: `WheneverCreatureDies{controller: Some(You)}` → `LoseLife{EachOpponent, 1}` — matches "creature you control dies → each opponent loses 1."
  - Death trigger 2: `WheneverCreatureDies{controller: Some(Opponent)}` → `GainLife{Controller, 1}` — matches "creature an opponent controls dies → you gain 1." Correct recipient (Controller = you).
- **Verdict**: PASS

## Card 5: Scion of Draco
- **Oracle match**: YES
- **Types match**: YES (Artifact Creature — Dragon, 4/4)
- **Mana cost match**: YES ({12})
- **DSL correctness**: YES
- **Findings**: none. Domain cost reduction via `self_cost_reduction: BasicLandTypes { per: 2 }` (def L44). Five color→keyword statics via `CreaturesYouControlWithColor(color)` + `AddKeyword`; each color mapping matches oracle: White→Vigilance, Blue→Hexproof, Black→Lifelink, Red→FirstStrike, Green→Trample. All five load-bearing and correct. Flying keyword present.
- **Verdict**: PASS

## Card 6: Sarkhan Vol
- **Oracle match**: YES
- **Types match**: YES (Legendary Planeswalker — Sarkhan, loyalty 4)
- **Mana cost match**: YES ({2}{R}{G})
- **DSL correctness**: YES
- **Findings**: none.
  - +1: `ModifyBoth(1)` + `AddKeywords(Haste)` on `CreaturesYouControl`, UEOT — matches anthem+haste.
  - −2: `GainControl{target 0, UntilEndOfTurn}` + `UntapPermanent{target 0}` + `AddKeywords(Haste)` — matches "gain control UEOT, untap, gains haste."
  - −6: five 4/4 red Dragon flying tokens (`count: Fixed(5)`). Correct.
- **Verdict**: PASS

## Card 7: Elven Chorus
- **Oracle match**: YES
- **Types match**: YES (Enchantment)
- **Mana cost match**: YES ({3}{G})
- **DSL correctness**: YES (marker correct)
- **Findings**:
  - Clauses 1+2 (look-at-top + cast creatures from top) implemented via `StaticPlayFromTop{CreaturesOnly, look_at_top: true}`.
  - Clause 3 ("Creatures you control have '{T}: Add one mana of any color.'") correctly left unauthored: verified the `any_color` ManaAbility stub in `rules/mana.rs` (see header) — wiring it would ship wrong game state (always colorless), a W5/known-wrong outcome, not merely incomplete. `enduring_vitality.rs` confirmed still `partial`, so not valid Complete precedent.
- **Verdict**: PASS (partial marker valid and well-reasoned)

## Card 8: Olivia Voldaren
- **Oracle match**: YES
- **Types match**: YES (Legendary Creature — Vampire, 3/3)
- **Mana cost match**: YES ({2}{B}{R})
- **DSL correctness**: PARTIAL — see F1
- **Findings**:
  - **{1}{R} ability — CORRECT**: `DealDamage{target 0, 1}` + `AddSubtypes(Vampire)` at `TypeChange` layer `Indefinite` (in-addition-to, no removal — matches "becomes a Vampire in addition to its other types"; CR 611.2c indefinite) + `AddCounter{Source, +1/+1}`. Target `TargetCreatureWithFilter{exclude_self: true}` — "another target creature" correctly excludes Olivia (def L60-63). Damage recipient and counter recipient both correct.
  - **F1 (MEDIUM): {3}{B}{B} gain-control uses wrong duration.** Oracle: "Gain control of target Vampire **for as long as you control Olivia Voldaren**." Def L76-79 uses `EffectDuration::WhileSourceOnBattlefield`. These diverge precisely when an opponent gains control of Olivia while she remains on the battlefield: the borrowed Vampire should return to its owner's control (you no longer control Olivia) but under `WhileSourceOnBattlefield` it does not. This ships **wrong game state** in that (real, if narrow) multiplayer scenario. Verified no `EffectDuration` variant expresses "as long as you control source" (`continuous_effect.rs` L44-64) — this is blocked on a missing primitive, not authorable-as-is.
- **Verdict**: **DEMOTE** — mark non-Complete. Recommend `partial` (or `known_wrong`) with note: "'for as long as you control Olivia Voldaren' modeled as WhileSourceOnBattlefield; blocked on a missing EffectDuration::WhileYouControlSource — diverges (borrowed Vampire fails to return) if an opponent gains control of Olivia while she stays on the battlefield. The {1}{R} ability is fully correct." Adjudication rationale: under W6's strict no-wrong-state policy, an implemented-but-incorrect clause is not an "acceptable idiom"; the taxonomy (ships wrong state, does not merely omit) leans `known_wrong`, but `partial` with the missing-primitive note is defensible since it is correct in the common case and the divergence requires an external gain-control effect.

## Card 9: Jagged-Scar Archers
- **Oracle match**: YES
- **Types match**: YES (Creature — Elf Archer, */*)
- **Mana cost match**: YES ({1}{G}{G})
- **DSL correctness**: YES
- **Findings**: none. `power: None, toughness: None` for `*/*` CDA (KI-5 satisfied, def L20-21). `CdaPowerToughness` P and T each = `PermanentCount{creature+Elf, Controller}` (counts self, correct). `{T}` ability: `DealDamage{target 0, PowerOf(Source)}`, target `TargetCreatureWithFilter{has_keywords: Flying}` — "target creature with flying." Correct.
- **Verdict**: PASS

## Card 10: Goblin Cratermaker
- **Oracle match**: NO (name-vs-"This creature", see F1)
- **Types match**: YES (Creature — Goblin Warrior, 2/2)
- **Mana cost match**: YES ({1}{R})
- **DSL correctness**: N/A (correctly marked known_wrong)
- **Findings**:
  - Marker JUSTIFIED: modal ACTIVATED ability; `AbilityDefinition::Activated` has no `modes` field (confirmed header), so only gated `Effect::Choose` is available — always resolves mode 0 and still demands a legal mode-1 target that never executes → wrong game state. Secondary defect (mode-1 filter is bare `non_land: true`, missing colorless restriction) noted in marker. Cannot be Complete until a modal-activated primitive exists.
  - F1 (LOW, KI-18): Def oracle L19-20 writes "Goblin Cratermaker deals 2 damage"; Scryfall templates as "This creature deals 2 damage to target creature." Cosmetic.
- **Verdict**: PASS (known_wrong marker valid)

## Card 11: Radha, Heart of Keld
- **Oracle match**: YES
- **Types match**: YES (Legendary Creature — Elf Warrior, 3/3)
- **Mana cost match**: YES ({1}{R}{G})
- **DSL correctness**: YES
- **Findings**:
  - **NOTE on the brief**: the brief's "conditional +2/+0 first-strike (≤1 card in hand)" description belongs to *Radha, Heir to Keld* (a different card), not *Radha, Heart of Keld*. The MCP oracle for this card is: "During your turn, Radha has first strike. / You may look at the top card of your library any time, and you may play lands from the top of your library. / {4}{R}{G}: Radha gets +X/+X until end of turn, where X is the number of lands you control." The def implements exactly this and correctly does NOT have any +2/+0 or hand-size clause.
  - "During your turn, first strike": `Static AddKeyword(FirstStrike)` on `Source`, `condition: Some(IsYourTurn)`. Correct.
  - Look-at-top + play lands from top: `StaticPlayFromTop{LandsOnly, look_at_top: true}`. Correct.
  - `{4}{R}{G}`: `ModifyBothDynamic{PermanentCount(land, Controller)}` on `Source`, UEOT. X = lands you control, locked at resolution. Correct — this IS the true third ability (author's RevealAndRoute correction is right).
- **Verdict**: PASS

## Card 12: Keeper of Fables
- **Oracle match**: YES
- **Types match**: YES (Creature — Cat, 4/5)
- **Mana cost match**: YES ({3}{G}{G})
- **DSL correctness**: YES
- **Findings**:
  - Trigger `WhenOneOrMoreCreaturesYouControlDealCombatDamageToPlayer{filter: creature + exclude_subtypes:[Human]}` → `DrawCards{Controller, 1}`. "non-Human creatures you control" via `exclude_subtypes` (controller-you baked into the trigger variant name). Batch semantics (draw once) correct.
  - F1 (LOW): verify `exclude_subtypes` is actually honored in this trigger's filter matching — CLAUDE.md warns several `TargetFilter` fields are silently ignored by `matches_filter`. If ignored here, the card would draw on Human combat damage too (would be wrong-state). Recommend a spot-check; not demoting absent confirmation of a gap.
- **Verdict**: PASS (with F1 verification note)

---

## Summary
- **Cards with issues**: Olivia Voldaren (F1 MEDIUM — DEMOTE), Aspect of Hydra (LOW reminder text), Goblin Cratermaker (LOW oracle wording; marker already known_wrong), Keeper of Fables (LOW verify exclude_subtypes)
- **Clean cards**: Craterhoof Behemoth, Devilish Valet, The Meathook Massacre, Scion of Draco, Sarkhan Vol, Elven Chorus (partial marker valid), Jagged-Scar Archers, Radha Heart of Keld
- **Verdicts**: 10 PASS, 1 DEMOTE (Olivia), 1 PASS-known_wrong (Goblin marker valid)
- **Meathook**: X threading CONFIRMED — stays Complete.
- **Elven Chorus / Goblin marker**: both justified against verified engine stubs.
- **Radha**: def matches real oracle; brief's "+2/+0" description was a different card (Radha, Heir to Keld).
