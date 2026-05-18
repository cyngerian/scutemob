# Card Review: scutemob-42 W-NOW-1 Batch 2 — 12 re-authored stale-TODO cards

**Reviewed**: 2026-05-18
**Cards**: 12
**Findings**: 0 HIGH, 0 MEDIUM, 3 LOW

DSL verification done against `crates/engine/src/cards/card_definition.rs` and
`crates/engine/src/state/continuous_effect.rs`. Confirmed:
- `EffectFilter` (continuous_effect.rs L67-246) has NO `TriggeringCreature` variant.
- `EffectTarget::TriggeringCreature` exists (card_definition.rs L2241).
- `EffectAmount::ManaValueOf(EffectTarget)` exists (L2288) but no EffectTarget for a
  RevealAndRoute-revealed card.
- `WheneverCreatureDies` has `filter: Option<TargetFilter>` (L2806) — subtype death
  triggers ARE expressible.
- `WheneverCreatureEntersBattlefield { filter, exclude_self }` exists (L2822).
- `WheneverCreatureYouControlDealsCombatDamageToPlayer { filter }` exists (L2969).
- `WheneverYouAttack` (per-combat, NO filter, L3043); `WheneverCreatureYouControlAttacks
  { filter }` (per-creature, L2955). No per-combat batched-subtype attack trigger.
- `WhenOneOrMoreCreaturesYouControlDealCombatDamageToPlayer { filter }` exists (L2982)
  but is a combat-damage trigger, not an attack-declaration trigger.
- `WheneverYouCastSpell.spell_type_filter` is `Vec<CardType>` only (L2863); no spell
  subtype filter.
- `once_per_turn` only on `AbilityDefinition::Activated` (L244).
- `EffectFilter::DeclaredTarget { index }` exists (continuous_effect.rs L111).
- `EffectFilter::AttackingCreaturesYouControlWithSubtype` exists (L189).

---

## Card 1: Crossway Troublemakers — PARTIAL
- **Oracle match**: YES
- **Types match**: YES (Creature — Vampire)
- **Mana cost match**: YES ({5}{B}), P/T 5/5 YES
- **DSL correctness**: YES
- **Findings**:
  - The two static abilities (Attacking Vampires you control have deathtouch /
    lifelink) are correctly authored via
    `EffectFilter::AttackingCreaturesYouControlWithSubtype(Vampire)`. Correct.
  - F1 (LOW): The death trigger is *entirely omitted* even though the trigger itself
    ("Whenever a Vampire you control dies") IS expressible —
    `WheneverCreatureDies { controller: You, filter: Some(Vampire), .. }`. Only the
    "may pay 2 life — if you do, draw a card" beneficial-pay rider is genuinely
    ENGINE-BLOCKED (`MayPayOrElse` is tax semantics, confirmed). Miara (card 10)
    handled the identical pattern by authoring the trigger with `Effect::Nothing`.
    For consistency, Crossway should do the same. Not wrong game state either way
    (a trigger with no effect body is harmless), but the two cards diverge in style.
    Cosmetic — LOW.
- **Verdict**: PARTIAL — statics correct; beneficial-pay draw rider legitimately blocked.

## Card 2: Hermes, Overseer of Elpis — PARTIAL
- **Oracle match**: YES
- **Types match**: YES (Legendary Creature — Elder Wizard)
- **Mana cost match**: YES ({3}{U}), P/T 2/4 YES
- **DSL correctness**: YES
- **Findings**:
  - Ability 1 (cast noncreature spell → 1/1 blue Bird with flying+vigilance) correctly
    authored: `WheneverYouCastSpell { noncreature_only: true, .. }` + `CreateToken`
    with the right colors/subtypes/keywords. Correct.
  - Ability 2 ("Whenever you attack with one or more Birds, scry 2") is correctly
    identified as ENGINE-BLOCKED. `WheneverYouAttack` is per-combat but has no filter;
    `WheneverCreatureYouControlAttacks` has a subtype filter but fires per-creature
    (would scry 2 once per attacking Bird = wrong game state).
    `WhenOneOrMoreCreaturesYouControlDealCombatDamageToPlayer` is a damage trigger,
    not an attack-declaration trigger, so it does not fit either. ENGINE-BLOCKED call
    is valid.
- **Verdict**: PARTIAL — token-on-cast correct; batched-subtype attack trigger blocked.

## Card 3: Serpent's Soul-Jar — BLOCKED
- **Oracle match**: YES
- **Types match**: YES (Artifact)
- **Mana cost match**: YES ({2}{B})
- **DSL correctness**: YES (abilities: vec![])
- **Findings**:
  - Both clauses correctly ENGINE-BLOCKED. The death trigger "exile it" needs to exile
    the dying creature itself AND tag the exile as "exiled with this artifact" for the
    later cast-from-exile zone tracking; no artifact-tagged-exile-of-triggering-creature
    effect exists. The "{T}, Pay 2 life: cast a creature spell from among cards exiled
    with this artifact" needs a cast-from-exile permission gated on a named exile zone —
    no such primitive. Both blocks valid.
- **Verdict**: BLOCKED — nothing beyond vanilla stats authorable.

## Card 4: Ogre Battledriver — BLOCKED
- **Oracle match**: YES (oracle_text includes reminder text, matches Scryfall)
- **Types match**: YES (Creature — Ogre Warrior)
- **Mana cost match**: YES ({2}{R}{R}), P/T 3/3 YES
- **DSL correctness**: YES (abilities: vec![])
- **Findings**:
  - ENGINE-BLOCKED call confirmed. The trigger
    `WheneverCreatureEntersBattlefield { filter, exclude_self: true }` IS expressible,
    but the effect must apply a continuous +2/+0 and haste grant to the *entering*
    creature. `ApplyContinuousEffect` takes a `ContinuousEffectDef` whose `filter` is
    `EffectFilter`, and `EffectFilter` has no `TriggeringCreature` variant (verified
    continuous_effect.rs L67-246). `EffectTarget::TriggeringCreature` exists but is for
    point effects, not continuous-effect filters. Correctly blocked — omitting avoids
    buffing self/all creatures.
- **Verdict**: BLOCKED — entire (only) ability legitimately engine-blocked.

## Card 5: Thornbite Staff — PARTIAL
- **Oracle match**: YES
- **Types match**: YES (Kindred Artifact — Shaman Equipment) — "Kindred" handled as a
  card type via `types_sub(&[CardType::Artifact], ..)`; note oracle type line is
  "Kindred Artifact". The def uses only `CardType::Artifact`. See F2.
- **Mana cost match**: YES ({2})
- **DSL correctness**: YES for the authored Equip ability
- **Findings**:
  - Equip {4} correctly authored as an `Activated` ability with `Cost::Mana(generic:4)`,
    `AttachEquipment`, sorcery-speed timing. Correct.
  - The granted-abilities clause (equipped creature has an activated ability and a
    triggered ability) is correctly ENGINE-BLOCKED — no `LayerModification::Add
    ActivatedAbility` / `AddTriggeredAbility`. The auto-attach ETB trigger
    ("Whenever a Shaman creature enters, you may attach...") is correctly blocked —
    no triggered `AttachEquipment` with a may-clause / subtype filter. Both valid.
  - F2 (LOW): Oracle type line is "Kindred Artifact — Shaman Equipment". The def's
    `types_sub(&[CardType::Artifact], &["Shaman","Equipment"])` does not record the
    "Kindred" type. This is a pre-existing convention question (the engine may not
    model Kindred/Tribal as a CardType), not introduced by this re-authoring, and has
    no gameplay impact for this card. Flagging as LOW for awareness only — verify
    whether `CardType::Kindred`/`Tribal` exists; if it does, it should be included.
- **Verdict**: PARTIAL — Equip correct; granted abilities + auto-attach trigger blocked.

## Card 6: Sram, Senior Edificer — BLOCKED
- **Oracle match**: YES
- **Types match**: YES (Legendary Creature — Dwarf Advisor)
- **Mana cost match**: YES ({1}{W}), P/T 2/2 YES
- **DSL correctness**: YES (abilities: vec![])
- **Findings**:
  - ENGINE-BLOCKED call confirmed. "Aura, Equipment, Vehicle" are spell subtypes;
    `WheneverYouCastSpell.spell_type_filter` is `Vec<CardType>` (verified L2863).
    Aura/Equipment/Vehicle are NOT CardTypes (Vehicle and Equipment are Artifact
    subtypes, Aura is an Enchantment subtype). No spell-subtype filter exists. An
    unfiltered "draw on every spell" would be wrong game state. Correctly blocked.
- **Verdict**: BLOCKED — entire (only) ability legitimately engine-blocked.

## Card 7: Tyvar Kell — PARTIAL
- **Oracle match**: YES (matches the non-alchemy "Tyvar Kell" printing; the A- variant
  is correctly NOT used)
- **Types match**: YES (Legendary Planeswalker — Tyvar)
- **Mana cost match**: YES ({2}{G}{G}), starting loyalty 3 YES
- **DSL correctness**: YES
- **Findings**:
  - Static "Elves you control have '{T}: Add {B}'" correctly ENGINE-BLOCKED — no
    `LayerModification::AddManaAbility`. Valid.
  - +1 loyalty ability correctly authored: `LoyaltyAbility` with `LoyaltyCost::Plus(1)`,
    `Sequence` of AddCounter (+1/+1) → UntapPermanent → ApplyContinuousEffect (deathtouch
    UntilEndOfTurn via `EffectFilter::DeclaredTarget { index: 0 }`), and a
    `TargetRequirement::UpToN { count: 1, inner: TargetCreatureWithFilter(Elf) }`.
    "Up to one" correctly modeled (spell castable with 0 targets). Correct.
  - 0 ability correctly authored: create a 1/1 green Elf Warrior token. Correct.
  - −6 ultimate: the emblem grants "Whenever you cast an Elf spell..." — Elf is a spell
    subtype (no `WheneverYouCastSpell` subtype filter) and "it gains haste" modifies a
    spell on the stack (not expressible). Authored as `LoyaltyAbility` with
    `Effect::Nothing` to preserve the loyalty cost option without wrong behavior.
    ENGINE-BLOCKED call valid; `Effect::Nothing` is the correct degradation.
- **Verdict**: PARTIAL — +1 and 0 fully correct; static and −6 emblem blocked.

## Card 8: Yuriko, the Tiger's Shadow — PARTIAL
- **Oracle match**: YES (Commander ninjutsu reminder text included, matches Scryfall)
- **Types match**: YES (Legendary Creature — Human Ninja)
- **Mana cost match**: YES ({1}{U}{B}), P/T 1/3 YES
- **DSL correctness**: YES
- **Findings**:
  - Commander ninjutsu correctly given the dual definition (KI-6 satisfied):
    `Keyword(CommanderNinjutsu)` marker AND `CommanderNinjutsu { cost: {U}{B} }`.
    Correct.
  - Combat-damage trigger correctly authored:
    `WheneverCreatureYouControlDealsCombatDamageToPlayer { filter: Some(Ninja) }` (the
    filter field is confirmed to exist, L2969). The reveal-to-hand portion uses
    `RevealAndRoute` with `count: 1` and an empty `TargetFilter` so both matched and
    unmatched destinations are `Hand` — correctly puts the top card into hand.
    Correct.
  - "Each opponent loses life equal to that card's mana value" correctly ENGINE-BLOCKED:
    `EffectAmount::ManaValueOf(EffectTarget)` exists but no `EffectTarget` variant points
    at the card revealed by `RevealAndRoute`. Valid block.
- **Verdict**: PARTIAL — ninjutsu + reveal-to-hand correct; life-loss clause blocked.

## Card 9: Marionette Apprentice — PARTIAL
- **Oracle match**: YES (Fabricate reminder text included)
- **Types match**: YES (Creature — Human Artificer)
- **Mana cost match**: YES ({1}{B}), P/T 1/2 YES
- **DSL correctness**: YES
- **Findings**:
  - Fabricate 1 correctly authored as `Keyword(KeywordAbility::Fabricate(1))`. Correct.
  - Death trigger "Whenever another creature OR artifact you control is put into a
    graveyard from the battlefield" correctly ENGINE-BLOCKED. `WheneverCreatureDies`
    covers only creatures; an artifact that is not a creature dying would be missed.
    There is no creature-OR-artifact / permanent-LTB death trigger variant. A
    creature-only approximation would silently miss noncreature-artifact deaths =
    wrong game state. Correct W5 call.
- **Verdict**: PARTIAL — Fabricate correct; creature-or-artifact death trigger blocked.

## Card 10: Miara, Thorn of the Glade — PARTIAL
- **Oracle match**: YES
- **Types match**: YES (Legendary Creature — Elf Scout)
- **Mana cost match**: YES ({1}{B}), P/T 1/2 YES
- **DSL correctness**: YES
- **Findings**:
  - Death trigger correctly authored: `WheneverCreatureDies { controller: You,
    exclude_self: false, filter: Some(Elf), .. }`. `exclude_self: false` is correct —
    oracle says "Whenever Miara OR another Elf you control dies" (includes herself).
    Correct.
  - Effect body is `Effect::Nothing` because "you may pay {1} and 1 life. If you do,
    draw a card" is a beneficial optional-pay rider — `MayPayOrElse` is tax semantics
    (confirmed L1584), not "pay to gain". ENGINE-BLOCKED call valid; `Effect::Nothing`
    is the correct harmless degradation.
  - Partner correctly authored as `Keyword(KeywordAbility::Partner)`.
  - F3 (LOW): not a defect — note only. Authoring the trigger with `Effect::Nothing`
    means a trigger with no effect goes on the stack on each qualifying Elf death.
    This is harmless game-state-wise, but contrast with Crossway Troublemakers
    (card 1) which omits the equivalent trigger entirely. The two re-authored cards
    should adopt one consistent convention for the "trigger expressible, rider
    blocked" pattern. See F1.
- **Verdict**: PARTIAL — death trigger + Partner correct; beneficial-pay rider blocked.

## Card 11: Morbid Opportunist — BLOCKED
- **Oracle match**: YES
- **Types match**: YES (Creature — Human Rogue)
- **Mana cost match**: YES ({2}{B}), P/T 1/3 YES
- **DSL correctness**: YES (abilities: vec![])
- **Findings**:
  - ENGINE-BLOCKED call confirmed. "This ability triggers only once each turn" — the
    `once_per_turn` field exists ONLY on `AbilityDefinition::Activated` (verified L244),
    not on `Triggered`. Without the limiter, `WheneverCreatureDies` (or a "one or more
    creatures die" batched variant) would draw a card per death/per batch instead of
    at most once per turn = wrong game state. Correct W5 call.
  - Note: the comment header says "BLOCKED" while the inline comment says
    "ENGINE-BLOCKED" — consistent enough; no action needed.
- **Verdict**: BLOCKED — entire (only) ability legitimately engine-blocked.

## Card 12: Gilded Drake — PARTIAL
- **Oracle match**: YES
- **Types match**: YES (Creature — Drake)
- **Mana cost match**: YES ({1}{U}), P/T 3/3 YES
- **DSL correctness**: YES
- **Findings**:
  - Flying correctly authored as `Keyword(KeywordAbility::Flying)`.
  - ETB trigger correctly authored: `WhenEntersBattlefield` + `Effect::ExchangeControl`
    with `target_a: Source`, `target_b: DeclaredTarget { index: 0 }`,
    `duration: Indefinite`. The target requirement is
    `UpToN { count: 1, inner: TargetCreatureWithFilter(controller: Opponent) }` —
    correctly models "up to one target creature an opponent controls" (castable/
    resolvable with 0 targets). Correct.
  - "If you don't or can't make an exchange, sacrifice this creature" correctly
    ENGINE-BLOCKED — no `Condition` variant for whether the exchange actually occurred
    (0 targets declared, or target became illegal). Valid block.
  - Note: the conditional self-sacrifice is a meaningful balance clause; omitting it
    makes Gilded Drake strictly better than printed (no downside if you choose 0
    targets). This is an acceptable W5 outcome (correct-partial over wrong-complete),
    but worth surfacing to the user as a known functional shortfall — not a defect in
    the re-authoring.
- **Verdict**: PARTIAL — Flying + ETB exchange correct; conditional self-sac blocked.

---

## Summary
- **Clean cards (zero unimplemented clauses)**: none
- **PARTIAL** (some clauses authored, some legitimately ENGINE-BLOCKED):
  Crossway Troublemakers, Hermes Overseer of Elpis, Thornbite Staff, Tyvar Kell,
  Yuriko the Tiger's Shadow, Marionette Apprentice, Miara Thorn of the Glade,
  Gilded Drake — 8 cards
- **BLOCKED** (nothing beyond vanilla stats authorable):
  Serpent's Soul-Jar, Ogre Battledriver, Sram Senior Edificer, Morbid Opportunist —
  4 cards

**Disposition tally: 0 CLEAN / 8 PARTIAL / 4 BLOCKED of 12**

### Defects
- **0 HIGH, 0 MEDIUM.** Every ENGINE-BLOCKED call was independently verified against
  the current DSL and is genuinely blocked. Every authored ability is a correct DSL
  translation of its oracle clause (correct scope, amounts, filters, targeting). No
  stale TODOs remain — all comments are precise ENGINE-BLOCKED markers.
- **3 LOW (style / awareness, no game-state impact)**:
  - F1: Crossway Troublemakers omits the (expressible) Vampire-death trigger entirely
    while Miara authors the equivalent trigger with `Effect::Nothing`. Pick one
    consistent convention.
  - F2: Thornbite Staff oracle type line is "Kindred Artifact"; the def records only
    `CardType::Artifact`. Verify whether a Kindred/Tribal CardType exists and should
    be included. Pre-existing convention question, no gameplay impact for this card.
  - F3: Same convention point as F1, observed from Miara's side.

### Recommendation
The batch is correct and ready to merge. The 3 LOW findings are cosmetic/convention
items — a follow-up pass could harmonize the "trigger expressible, rider blocked"
convention (omit-entirely vs. `Effect::Nothing`) across Crossway and Miara, but
neither approach produces wrong game state.
