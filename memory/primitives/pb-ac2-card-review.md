# PB-AC2 Backfill Card Review

**Reviewed**: 2026-07-07
**Cards**: 20 (12 CLEAN + 8 PARTIAL)
**Findings**: 0 HIGH, 0 MEDIUM, 1 LOW (+ 1 informational engine note)

Reviewer verified every card's oracle text against the mtg-rules MCP, confirmed the
`MayPayThenEffect` / `CounterUnlessPays` DSL usage, and traced the referenced DSL
fields/variants/trigger-conditions to source in `card_definition.rs` and `effects/mod.rs`.

---

## Informational (engine primitive, NOT a card defect)

`Effect::CounterUnlessPays { target, cost: _ }` in `effects/mod.rs:3025` **ignores the
cost and always counters** (delegates unconditionally to `Effect::CounterSpell`). This is
the documented, intentional deterministic fallback for PB-AC2 (comment L3018-3024: "the
deterministic path always counters ... the payer never has an incentive to voluntarily tax
themselves without interactive choice"; `cost` retained for M10 interactive payment /
hashing / display). Consequence: all six counter-tax card defs (Mana Leak, Mana Tithe,
Spell Pierce, Flusterstorm, Make Disappear, Izzet Charm mode 0, and Stubborn Denial's
if_false branch) currently behave as unconditional counters until M10.

This is consistent across the whole engine (same class as Scry/Surveil deterministic
fallbacks) and was accepted in the PB-AC2 primitive review. It is a primitive-behavior
matter, not a card-authoring defect — the card definitions correctly select the primitive.
Noted here only so the counter-tax cards are not later mistaken for wrong game state.

Similarly `MayPayThenEffect` (`effects/mod.rs:2999`) resolves the optional cost via
`try_pay_optional_cost` and correctly rebinds `ctx.controller` to the actual payer for the
`then` branch (so a non-Controller payer's DrawCards benefits the payer). All
`MayPayThenEffect` card defs reviewed use `payer: PlayerTarget::Controller`, which is
correct for every "you may pay ... draw a card" rider here.

---

## CLEAN cards (12) — all correct

### Crossway Troublemakers — CLEAN
Oracle "Whenever **a** Vampire you control dies" (not "another"). `exclude_self: false`
with `has_subtype: Vampire`, `controller: You` is correct (source is itself a Vampire and
is included). Both attacking-Vampire static grants (deathtouch, lifelink) intact and
unaffected. Cost `PayLife(2)` -> `DrawCards(1)` matches. Oracle text field matches MCP.

### Miara, Thorn of the Glade — CLEAN
Oracle "Whenever Miara or another Elf you control dies". Elf subtype filter + `exclude_self:
false` correctly covers both Miara (an Elf) and other Elves. Cost `Sequence([Mana{1},
PayLife(1)])` matches "{1} and 1 life". `KeywordAbility::Partner` kept. Legendary supertype
+ Elf Scout types + 1/2 correct.

### Hazoret's Monument — CLEAN (fix confirmed)
Previously an unconditional draw (wrong). Now `MayPayThenEffect { cost: DiscardCard, then:
DrawCards(1) }` — draw only fires if a card is discarded. Correct game state.
`SpellCostFilter::ColorAndCreature(Red)` reducer for "Red creature spells cost {1} less"
intact. Legendary Artifact supertype correct.

### Tainted Observer — CLEAN
Oracle has no ETB draw (task's "+ETB draw?" is a non-issue). `WheneverCreatureEntersBattlefield`
with `controller: You` + `exclude_self: true` matches "another creature you control enters".
`MayPayThenEffect { cost: Mana{2}, then: Proliferate }` correct. Flying + Toxic(1) kept.
Oracle text field (incl. reminder text) matches MCP.

### Springbloom Druid — CLEAN
ETB `MayPayThenEffect { cost: Sacrifice(Land filter), then: [SearchLibrary basic tapped,
SearchLibrary basic tapped, Shuffle] }`. Two single-card `SearchLibrary` effects model "up
to two basic land cards" (SearchLibrary has no count field; established engine pattern);
single trailing `Shuffle` gives correct end state. Source is not a land, so the sacrifice
filter cannot hit itself.

### Nadir Kraken — CLEAN
`WheneverYouDrawACard` -> `MayPayThenEffect { cost: Mana{1}, then: [AddCounter(+1/+1) on
Source, CreateToken 1/1 blue Tentacle] }`. Counter + token gated on paying {1}. Card is a
fixed 2/3 (no CDA / no `*/*`), so `power/toughness: Some(2/3)` correct; task's "P/T that
scales?" is a non-issue.

### Mana Leak / Mana Tithe — CLEAN
`CounterUnlessPays { target: DeclaredTarget{0}, cost: Mana{3} / Mana{1} }`, target
`TargetSpellWithFilter(default)` (any spell). Mana costs, colors ({1}{U} / {W}) correct.

### Spell Pierce — CLEAN
`cost: Mana{2}`, target filter `non_creature: true` matches "noncreature spell". {U} correct.

### Flusterstorm — CLEAN
`cost: Mana{1}`, target filter `has_card_types: vec![Instant, Sorcery]`. Verified
`has_card_types` is OR semantics (card_definition.rs:2623) — matches "instant or sorcery".
`KeywordAbility::Storm` kept. Oracle text (incl. Storm reminder) matches MCP.

### Make Disappear — CLEAN (see LOW-1 for reminder-text nit)
`cost: Mana{2}`, any spell. `KeywordAbility::Casualty(1)` kept.

### Izzet Charm — CLEAN
Modal (min 1 / max 1). Mode 0 `CounterUnlessPays cost Mana{2}` target[0] `non_creature: true`;
mode 1 `DealDamage 2` to target[1] creature; mode 2 `[DrawCards 2, DiscardCards 2]`. All
three modes match oracle; modes 1/2 unchanged from prior.

---

## PARTIAL cards (8) — authored portions correct, ENGINE-BLOCKED markers precise

### Stubborn Denial — fully authored, correct
Ferocious modeled as `Conditional { condition: YouControlPermanent(creature min_power 4),
if_true: CounterSpell, if_false: CounterUnlessPays cost Mana{1} }`. Verified
`Condition::YouControlPermanent` (3149), `min_power: Option<i32>` (2582), `Effect::CounterSpell`
(1339) all exist. Target filter `non_creature: true`. Correct game state (subject to the
CounterUnlessPays deterministic-counter note above).

### Ezuri, Stalker of Spheres — PARTIAL, marker precise
ETB `MayPayThenEffect { cost: Mana{3}, then: [Proliferate, Proliferate] }` — correct.
ENGINE-BLOCKED "Whenever you proliferate, draw a card": verified NO proliferate
`TriggerCondition` exists (only `Effect::Proliferate` @1749 and `WheneverRingTemptsYou`
@2973). Marker accurate.

### Mana Vault — PARTIAL, marker precise
`DoesNotUntap` keyword + upkeep `MayPayThenEffect { cost: Mana{4}, then: UntapPermanent(Source) }`
correct. Two ENGINE-BLOCKED: (1) no `AtBeginningOfYourDrawStep` TriggerCondition — verified
none exists; (2) `{T}: Add {C}{C}{C}` omitted under W5 because its real downside (draw-step
ping while tapped) can't be modeled without the draw-step trigger — sound reasoning (free
mana without the pain = wrong game state).

### Call of the Ring — PARTIAL, marker precise
Upkeep `TheRingTemptsYou` authored. ENGINE-BLOCKED "Whenever you choose a creature as your
Ring-bearer ...": verified only `WheneverRingTemptsYou` exists (2973); no Ring-bearer-chosen
trigger. Marker accurate.

### Leaf-Crowned Visionary — PARTIAL, marker precise
"Other Elves you control get +1/+1" static authored (`OtherCreaturesYouControlWithSubtype(Elf)`,
`ModifyBoth(1)`). ENGINE-BLOCKED "Whenever you cast an Elf spell ...": `WheneverYouCastSpell`
has only `spell_type_filter: Vec<CardType>` + dynamic `chosen_subtype_filter` — no fixed
"Elf" subtype field; can't distinguish an Elf spell from any creature spell. Correctly
omitted rather than firing on every creature spell (W5). Marker accurate.

### Ruthless Technomancer — PARTIAL (fully blocked), marker precise
ETB blocked: `Cost::Sacrifice(filter)` has no "another"/exclude-self semantics (source is a
creature, could offer itself) AND no dynamic-power token-count plumbing. Activated ability
blocked: no variable-X sacrifice-count cost and no "power <= X" dynamic graveyard target
filter. Both genuine. `abilities: vec![]` — no wrong game state.

### Vampire Gourmand — PARTIAL (fully blocked), marker precise
Blocked on the same missing "another creature" (exclude-self) sacrifice-cost semantics; the
draw + can't-be-blocked rider is expressible but the self-sacrifice hazard is not
guardable. Omitted per W5. Cross-references wight_of_the_reliquary.rs (`Cost::SacrificeAnother`
does not exist). Marker accurate.

### Temur Sabertooth — PARTIAL (fully blocked), marker precise
Blocked because the optional action is a bounce ("return another creature to its owner's
hand"), not a `Cost` — `MayPayThenEffect` only wraps `Cost` variants; no
`Cost::ReturnPermanentToHand` exists. Correctly identified as NOT a MayPayThenEffect gap.
Marker accurate.

---

## Findings

- **LOW-1 (KI-18) — Make Disappear oracle_text reminder-text drift.**
  File `crates/engine/src/cards/defs/make_disappear.rs:13`. Def has Casualty reminder text
  "Casualty 1 (**As an additional cost to cast this spell,** you may sacrifice a creature
  with power 1 or greater. When you do, copy this spell**, and** you may choose a new target
  for the copy.)". Current Scryfall/MCP reads "Casualty 1 (**As you cast this spell,** you
  may sacrifice a creature with power 1 or greater. When you do, copy this spell **and** you
  may choose a new target for the copy.)". Reminder-text only; no gameplay impact.

## Summary
- Cards with issues: Make Disappear (LOW-1, oracle reminder-text drift only)
- Clean cards (no findings): all 19 others
- No HIGH, no MEDIUM. No wrong game state, no stale/imprecise ENGINE-BLOCKED markers, no
  oracle/type/mana/P-T mismatches beyond LOW-1.
