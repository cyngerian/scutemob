# PB-AC1 Card Review — counter / untap / once-per-turn primitives

**Reviewed**: 2026-07-07
**Cards**: 13 (8 CLEAN, 5 PARTIAL)
**Findings**: 0 HIGH, 0 MEDIUM, 4 LOW
**Verdict**: **clean** — all authored clauses produce correct game state; every `// ENGINE-BLOCKED`
marker was verified against engine source and is accurate. Only cosmetic oracle-text / fidelity
LOWs remain.

---

## CLEAN cards (markers gone; verified fully correct)

### Morbid Opportunist ({2}{B} Human Rogue 1/3)
- Oracle/type/cost/PT: MATCH.
- `WheneverCreatureDies { exclude_self: true }` + `once_per_turn: true` correctly encodes
  "one or more **other** creatures die ... triggers only **once each turn**". Draw to Controller. Correct.

### Mesmeric Orb ({2} Artifact)
- Oracle/type/cost: MATCH.
- `WheneverPermanentUntaps { filter: None }` (any permanent, any controller) →
  `MillCards { player: ControllerOf(TriggeringCreature) }` correctly routes the mill to
  "that permanent's controller". `once_per_turn: false`. Correct.

### Goblin Sharpshooter ({2}{R} Goblin 1/1)
- Type/cost/PT: MATCH. DSL correct: `DoesNotUntap` keyword; `WheneverCreatureDies
  { exclude_self: false }` (oracle "a creature dies" — any creature, incl. self) →
  `UntapPermanent(Source)`; `{T}: DealDamage 1 to any target`. Semantics correct.
- **F1 (LOW, oracle text)**: `oracle_text` uses self-name templating
  ("Goblin Sharpshooter doesn't untap...", "untap Goblin Sharpshooter",
  "Goblin Sharpshooter deals 1 damage") — current Scryfall oracle uses the modern
  "This creature ..." templating: *"This creature doesn't untap during your untap step. /
  Whenever a creature dies, untap this creature. / {T}: This creature deals 1 damage to any
  target."* Cosmetic only; behavior unaffected. `goblin_sharpshooter.rs:13`.

### Sharktocrab ({2}{G}{U} Shark Octopus Crab 4/4)
- Oracle/type/cost/PT: MATCH.
- Adapt keyword + `{2}{G}{U},{T}` activated ability with `Conditional
  { SourceHasNoCountersOfType(+1/+1) → AddCounter }` (mana spent regardless — matches ruling). Correct.
- `WhenCounterPlaced { counter: +1/+1, on_self: true }` → `Sequence[TapPermanent, PreventNextUntap]`
  targeting `TargetCreatureWithFilter { controller: Opponent }`. Correctly encodes "tap target
  creature an opponent controls. That creature doesn't untap during its controller's next untap step."
  `once_per_turn: false` (oracle has no once-each-turn clause). Correct.
- Note (informational, engine not authoring): oracle "one or more +1/+1 counters are put" is a
  once-per-**event** batch trigger; fidelity of the batching depends on the shared
  `WhenCounterPlaced` primitive, not on this card def. The author used the only/correct primitive.

### Dusk Legion Duelist ({1}{W} Vampire Soldier 2/2)
- Oracle/type/cost/PT: MATCH.
- Vigilance + `WhenCounterPlaced { +1/+1, on_self: true }` → Draw, `once_per_turn: true`.
  Correctly encodes "one or more ... put on this creature, draw a card. ... only once each turn." Correct.

### Whispering Wizard ({3}{U} Human Wizard 3/2)
- Oracle/type/cost/PT: MATCH.
- `WheneverYouCastSpell { noncreature_only: true }` → create 1/1 white Spirit w/ Flying,
  `once_per_turn: true`. Correct.

### Elvish Warmaster ({1}{G} Elf Warrior 2/2)
- Oracle/type/cost/PT: MATCH.
- Trigger `WheneverCreatureEntersBattlefield { filter: controller You + has_subtype Elf,
  exclude_self: true }` + `once_per_turn: true` → create 1/1 green Elf Warrior. Precise Elf-subtype
  filter (PB-AC0 wired `has_subtype` through the creature-ETB path). Correct — "other Elves you
  control enter ... only once each turn."
- Activated `{5}{G}{G}`: `ModifyBoth(2)` + `AddKeyword(Deathtouch)` over
  `CreaturesYouControlWithSubtype(Elf)`, UntilEndOfTurn. Correct (includes self, which is an Elf).

### Welcoming Vampire ({2}{W} Vampire 2/3)
- Oracle/type/cost/PT: MATCH.
- Flying + `WheneverCreatureEntersBattlefield { filter: controller You + max_power Some(2),
  exclude_self: true }` + `once_per_turn: true` → Draw. Correctly encodes "other creatures you
  control with power 2 or less ... only once each turn." Correct.

---

## PARTIAL cards (authored clause correct + ENGINE-BLOCKED markers verified accurate)

### Sky Hussar ({3}{W}{U} Human Knight 4/3)
- Oracle/type/cost/PT: MATCH.
- **Authored (correct)**: Flying; ETB `WhenEntersBattlefield → UntapAll { filter: has_card_type
  Creature, controller You }` — matches "untap all creatures **you control**" (filter.controller
  correctly `You`, not all creatures). Correct.
- **ENGINE-BLOCKED marker VALID** (`sky_hussar.rs:36-42`): Forecast's real cost is "Tap two untapped
  white and/or blue creatures you control, Reveal this card from hand" — a non-mana cost. Verified
  `AbilityDefinition::Forecast { cost: ManaCost, effect: Effect }` (`card_definition.rs:666`) accepts
  **only** a `ManaCost`, so this creature-tap+reveal cost is genuinely inexpressible. Marker text
  names the right blocker. A bare `Keyword(Forecast)` marker is present but harmless (advertises the
  keyword; no activatable ability — same practical result as blocked).

### Fathom Mage ({2}{G}{U} Human Wizard 1/1)
- Type/cost/PT: MATCH.
- **Authored (correct)**: Evolve keyword + `WhenCounterPlaced { +1/+1, on_self: true }` → Draw,
  `once_per_turn: false` (oracle "a +1/+1 counter is put ..." — no once-each-turn cap; per-counter).
  Correct.
- **F2 (LOW, oracle text)**: `oracle_text` says "put on Fathom Mage"; current Scryfall oracle uses
  "put on this creature". Cosmetic. `fathom_mage.rs:12`.
- **F3 (LOW, fidelity — TODO valid)**: oracle "you **may** draw a card" authored as a mandatory draw.
  Verified no optional-effect / `may`-draw wrapper exists in the effect DSL, so the TODO(optional-draw)
  is a real (non-stale) gap. Matches existing project convention (coastal_piracy, aesi). Matters only
  on the empty-library "would lose" edge case. `fathom_mage.rs:20-25`.

### Bear Umbra ({2}{G}{G} Enchantment — Aura)
- Oracle/type/cost: MATCH.
- **Authored (correct)**: `Enchant(Creature)`; +2/+2 via two `Static` PtModify effects
  (`ModifyPower(2)` + `ModifyToughness(2)`) over `AttachedCreature`; `UmbraArmor` keyword. Correct
  (functionally identical to a `ModifyBoth`; both statics on the enchanted creature only).
- **ENGINE-BLOCKED marker VALID** (`bear_umbra.rs:35-43`): the "Whenever this creature attacks, untap
  all lands you control" is a triggered ability **granted to the enchanted creature**. Verified no
  `LayerModification::AddTriggeredAbility` / `GrantTriggeredAbility` primitive exists — the same gap is
  cited by `diamond_pick_axe.rs`, `den_of_the_bugbear.rs`, `thornbite_staff.rs`, `dionus_...rs`. Marker
  correctly names the blocker and the eventual wiring (`Effect::UntapAll` over Lands you control).

### Mana Vault ({1} Artifact)
- Oracle/type/cost: MATCH.
- **Authored (correct)**: `DoesNotUntap` keyword only.
- **ENGINE-BLOCKED markers VALID** (`mana_vault.rs:7-16`):
  - "At the beginning of your upkeep, you may pay {4}. If you do, untap" — optional-mana-payment
    triggered ability; no optional-pay-in-trigger primitive. Valid.
  - "At the beginning of your draw step, if this artifact is tapped, deal 1 damage to you" — verified
    **no `AtBeginningOfYourDrawStep` TriggerCondition exists** (engine has only `AtBeginningOfYourUpkeep`).
    Genuinely inexpressible. Valid.
  - `{T}: Add {C}{C}{C}` correctly withheld under W5 policy: with the draw-step self-damage downside
    and the pay-{4}-untap escape both inexpressible, adding the mana ability would grant free colorless
    mana with none of the card's real cost (once tapped it stays tapped forever). Withholding is correct.

### Benefactor's Draught ({1}{G} Instant)
- Oracle/type/cost: MATCH.
- **`abilities: vec![]` — correct.** This is a single spell ability: "Untap all creatures" + a delayed
  "until end of turn, whenever a creature an opponent controls blocks, draw a card" + "Draw a card."
  Verified the delayed-trigger infra (`DelayedTriggerAction`) is a fixed enum (SacrificeObject /
  ExileObject / ReturnFromGraveyardToHand only) with no "delayed triggered **ability** carrying its own
  block sub-trigger" — so the block-on-opponent-block clause is genuinely inexpressible. Per W5, a single
  ability cannot be partially authored (dropping the delayed upside = wrong game state). Full block correct.
- **F4 (LOW, marker wording)**: marker says "no delayed-trigger-creation primitive in the DSL" — slightly
  overstated (limited delayed-trigger infra *does* exist for fixed one-shot actions). The conclusion is
  correct; only the wording could be tightened to "no delayed triggered-*ability*-with-sub-trigger
  primitive." Cosmetic. `benefactors_draught.rs:6-14`.

---

## Summary
- **Cards with issues**: Goblin Sharpshooter (F1 LOW), Fathom Mage (F2/F3 LOW), Benefactor's Draught (F4 LOW) — all cosmetic/fidelity.
- **Clean cards**: Morbid Opportunist, Mesmeric Orb, Sharktocrab, Dusk Legion Duelist, Whispering Wizard, Elvish Warmaster, Welcoming Vampire, Sky Hussar, Bear Umbra, Mana Vault.
- **No HIGH, no MEDIUM.** No wrong-game-state clause; no half-authored ability; no stale/now-expressible ENGINE-BLOCKED marker.

### Verdict: **clean**
- HIGH: none
- MEDIUM: none
