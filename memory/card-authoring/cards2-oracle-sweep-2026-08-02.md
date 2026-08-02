# CARDS-2 — clause-by-clause oracle sweep of 22 field-repaired defs

**Date**: 2026-08-02
**Task**: scutemob-181 (CARDS-2 second fix cycle), read-only audit
**Scope**: 22 defs named in the brief. `zulaport_cutthroat`, `akroma_angel_of_fury`,
`birchlore_rangers`, `prosperous_innkeeper`, `stock_up`, `green_suns_zenith` explicitly
excluded (concurrent agent); `braided_net`, `tyrranax_rex` already done.
**Method**: printed text from `mcp__mtg-rules__lookup_card` only. No def's own header
comment or `oracle_text` field was used as evidence about its card. All 22 are
single-faced, so the sqlite fallback was not needed. DSL claims were checked against
`crates/card-types/src/cards/card_definition.rs`,
`crates/card-types/src/state/replacement_effect.rs`, and `crates/engine/src/effects/mod.rs`.

**Clean: 11 of 22.**

**Findings**: 0 HIGH, 3 MEDIUM, 8 LOW (11 total across 11 defs).

---

## Headline negative result (the hypothesis under test)

The brief's worry was that `Completeness::Complete` defs in this batch implement abilities
their card does not have, or omit printed ones. **On these 22, that did not happen.**

Ten of the 22 are deck-legal `Complete` (`backup_agent`, `boon_satyr`, `changeling_hero`,
`glistener_elf`, `lonely_sandbar`, `necron_deathmark`, `overlord_of_the_hauntwoods`,
`saw_it_coming`, `whitemane_lion`, `windbrisk_heights` — the last five reach `Complete`
through `..Default::default()`, confirmed at `card_definition.rs:268`
`completeness: Completeness::Complete`). Every printed clause on all ten is authored, and
none of the ten authors a clause the card does not print. The one behavioural deviation on a
`Complete` def (`whitemane_lion`, F9) is a corpus-wide modelling convention, not a
CARDS-2 regression.

**Ability-embedded costs — all eight correct.** This was the dimension the field gate cannot
see and where three defects had already been found by hand this batch. Checked and matching
print: `boon_satyr` bestow `{3}{G}{G}`; `exalted_angel` morph `{2}{W}{W}`; `saw_it_coming`
foretell `{1}{U}`; `lonely_sandbar` cycling `{U}`; `overlord_of_the_hauntwoods` impending
`4—{1}{G}{G}`; `mystic_remora` cumulative upkeep `{1}` and the `{4}` opponent payment;
`windbrisk_heights` `{W}, {T}`; `the_world_tree` `{T}`. Zero defects.

What the sweep *did* find is a different failure mode: **stale in-file TODOs that name a DSL
gap the corpus has since closed, in three cases contradicted by the def's own
`Completeness` note two lines away.** Those are F1–F3 (MEDIUM) and F6/F7/F8/F10 (LOW).

---

## MEDIUM

### F1 — `chord_of_calling.rs`: stale DSL-gap TODO; "mana value X or less" IS expressible

- **File**: `crates/card-defs/src/defs/chord_of_calling.rs:25`
- **Severity**: MEDIUM (stale gap claim holding a card at `Partial`; def as written also
  searches for the wrong card pool)
- **Printed**: "Convoke … Search your library for a creature card with **mana value X or
  less**, put it onto the battlefield, then shuffle."
- **Def does**: `Effect::SearchLibrary` with
  `filter: TargetFilter { has_card_type: Some(CardType::Creature), ..Default::default() }` —
  **no mana-value cap at all**, so it fetches any creature in the library regardless of X.
  Carries `// TODO: "mana value X or less" — max_cmc should be XValue.` and
  `Completeness::partial("'mana value X or less' — max_cmc should be XValue")`.
- **The TODO is stale.** `TargetFilter.max_cmc_amount: Option<Box<EffectAmount>>` exists
  (`card_definition.rs:3086`) and its doc comment states it is "ONLY honored by the
  `Effect::SearchLibrary` executor (which has `ctx`)" — which is precisely this call site.
  The executor resolves it at `crates/engine/src/effects/mod.rs:3507`. `EffectAmount::XValue`
  exists (`card_definition.rs:2657`) and `card_definition.rs:1737` documents it as resolving
  "to the X value from the casting cost".
- **Repair, expressible today**: add
  `max_cmc_amount: Some(Box::new(EffectAmount::XValue))` to the filter. Precedent in
  corpus: `eldritch_evolution.rs`, `birthing_pod.rs`, `birthing_ritual.rs` all use the field.
  With that, the def implements every printed clause and can be promoted to `Complete`
  (Convoke is already `KeywordAbility::Convoke`, shipped in M6).

### F2 — `the_world_tree.rs`: inline TODO claims a gap its own note says does not exist

- **File**: `crates/card-defs/src/defs/the_world_tree.rs:39-40`
- **Severity**: MEDIUM (expressible printed ability unimplemented; two contradictory
  in-file claims)
- **Printed**: "As long as you control six or more lands, lands you control have
  \"{T}: Add one mana of any color.\""
- **Def does**: nothing. The clause is an inline
  `// TODO: … DSL gap: count_threshold + grant-ability-to-permanents.`
- **Contradiction**: the same file's `Completeness::partial` note (lines 44-49) says the
  opposite and is the accurate one: *"The six-lands static grant is NOT blocked —
  `LayerModification::AddManaAbility` (wired at `layers.rs:1193`) +
  `EffectFilter::LandsYouControl` + `ContinuousEffectDef.condition =
  YouControlNOrMoreWithFilter{count:6}` expresses it; rewire that clause."* The inline
  TODO was left un-updated when the note was written.
- **Repair**: implement per the note's own recipe (`AbilityDefinition::Static` with those
  three parts) and delete the stale inline TODO. The def stays `Partial` afterwards — the
  genuinely blocked clause is the `{W}{W}{U}{U}{B}{B}{R}{R}{G}{G}, {T}, Sacrifice` God
  tutor, since `Effect::SearchLibrary` has no count field (`card_definition.rs:1701-1719`,
  confirmed: `player`/`filter`/`reveal`/`destination`/`shuffle_before_placing`/
  `also_search_graveyard`, no `count`) and "any number of God cards" needs one.
- **Verified correct**: The World Tree is **not** Legendary (printed type line is plain
  `Land`); the def's `types(&[CardType::Land])` and its "(not Legendary)" header comment
  are both right.

### F3 — `flare_of_malice.rs`: `oracle_text` field is text the card does not print

- **File**: `crates/card-defs/src/defs/flare_of_malice.rs:1-3` (header) and `:16-18`
  (`oracle_text`)
- **Severity**: MEDIUM (not HIGH: `Completeness::known_wrong` keeps it out of `validate_deck`,
  so no deck-legal wrong behaviour — but the def *implements* the invented text)
- **Printed**: "You may sacrifice a nontoken black creature rather than pay this spell's mana
  cost. / **Each opponent** sacrifices **a creature or planeswalker with the greatest mana
  value among creatures and planeswalkers they control**."
- **Def's `oracle_text` says**: "…\nTarget opponent sacrifices a nonland permanent and loses
  2 life." — a different card. The header comment (line 3) repeats it.
- **Def does**: `Effect::Sequence([SacrificePermanents { player: DeclaredTarget{0}, count: 1,
  filter: None }, LoseLife { player: DeclaredTarget{0}, amount: 2 }])` with
  `targets: vec![TargetRequirement::TargetPlayer]` — i.e. it hits **one** player instead of
  each opponent, can hit **itself** (`TargetPlayer`, not `TargetOpponent`), sacrifices **any**
  permanent including lands (`filter: None`), and adds an **invented "loses 2 life"** clause.
- **Status**: the `known_wrong` note (lines 40-49) already describes all of this accurately
  and is the one place in the file telling the truth. **The remaining defect is that
  `oracle_text` and the header comment still carry the fictional text**, which is exactly the
  input that caused a previous pass to author three abilities onto the wrong card.
- **Repair**: replace `oracle_text` and the header comment with the printed text *now*, even
  though the abilities stay blocked. Genuine remaining blockers, both re-confirmed:
  (a) "greatest mana value among" is a selection rule, not a static `TargetFilter`;
  (b) the sacrifice-a-nontoken-black-creature alt cost (`TargetFilter` has no nontoken
  predicate). Note `TargetRequirement::TargetOpponent` *does* exist
  (`card_definition.rs:3032`) if a partial re-author is ever wanted.

---

## LOW

### F4 — `boon_satyr.rs`: `oracle_text` reminder text is a superseded printing

- **File**: `crates/card-defs/src/defs/boon_satyr.rs:30-33`
- **Printed**: "…It becomes a creature again if it's **not attached**.)"
- **Def has**: "…It becomes a creature again if it's **not attached to a creature**.)"
- Everything else on this def is correct and matches print clause for clause: `{1}{G}{G}`,
  `Enchantment Creature — Satyr`, 4/2, Flash, Bestow `{3}{G}{G}` (cost verified), and the
  `+4/+2` static pair on `EffectFilter::AttachedCreature`. The pattern matches the corpus's
  other bestow def (`springheart_nantuko.rs`, whose note confirms
  `AbilityDefinition::Bestow` + `EffectFilter::AttachedCreature` both exist and work).
  **Text fix only.**

### F5 — `saw_it_coming.rs`: `oracle_text` reminder text wrong word

- **File**: `crates/card-defs/src/defs/saw_it_coming.rs:16-18` (and header line 3)
- **Printed**: "Cast it on a **later** turn for its foretell cost.)"
- **Def has**: "Cast it on a **future** turn for its foretell cost.)"
- Def is otherwise fully correct and deck-legal `Complete`: `{1}{U}{U}` Instant, Foretell
  marker + `AbilityDefinition::Foretell { cost: {1}{U} }` (cost verified against print),
  `Effect::CounterSpell` on `TargetRequirement::TargetSpell`. **Text fix only.**

### F6 — `glacierwood_siege.rs`: `oracle_text` drops "target player" and uses the old self-reference

- **File**: `crates/card-defs/src/defs/glacierwood_siege.rs:18-21` (and header lines 2-3)
- **Printed**: "As **this enchantment** enters, choose Temur or Sultai. / • Temur — Whenever
  you cast an instant or sorcery spell, **target player mills** four cards. / • Sultai — You
  may play lands from your graveyard."
- **Def has**: "As **Glacierwood Siege** enters …" and "• Temur — … **mill** four cards."
  — two deviations: the pre-2021 self-reference templating, and a **dropped target**.
- **Knock-on**: the `Completeness::inert` note's proposed Temur repair
  ("`WheneverYouCastSpell{instant/sorcery}` + `MillCards(4)`") inherits the dropped target and
  would author an untargeted mill. Any future repair must add
  `TargetRequirement::TargetPlayer` and `player: PlayerTarget::DeclaredTarget { index: 0 }`.
- **Blocker claim re-verified as TRUE**: `ReplacementModification`
  (`crates/card-types/src/state/replacement_effect.rs:119`) has `ChooseCreatureType` and
  `ChooseColor` but **no mode-selection variant** — I read the full enum. The note's other
  claim also holds: `AbilityDefinition::StaticPlayFromGraveyard { filter: PlayFromTopFilter }`
  exists at `card_definition.rs:1115`, so the Sultai half is expressible. `abilities: vec![]`
  is the right call today. **Text fix only.**

### F7 — `prowess_of_the_fair.rs`: `Completeness` note's last sentence is stale, and contradicts the inline TODO

- **File**: `crates/card-defs/src/defs/prowess_of_the_fair.rs:21-24` (inline TODO) and
  `:26-32` (note)
- **Stale**: the note ends *"Type line also omits the Kindred card type."* — it does not.
  Line 16 reads `types_sub(&[CardType::Kindred, CardType::Enchantment], &["Elf"])`, matching
  the printed `Kindred Enchantment — Elf`. The field repair landed; the note did not follow.
- **Second contradiction**: the inline TODO (lines 21-24) claims *"DSL lacks a triggered
  condition with subtype + nontoken + controller filter"*, while the note two lines below says
  *"WheneverCreatureDies now supports controller/exclude_self/nontoken_only/filter
  (card_definition.rs:3048-3062) — the note's claimed gap is gone."* The note is correct; the
  inline TODO is the stale one.
- **The surviving blocker is real**, and it is neither of those two: printed says "another
  nontoken **Elf**", which includes noncreature Elves (Kindred cards — this very card is one),
  while `WheneverCreatureDies` is creature-only. `abilities: vec![]` stays right.
- Aside, not a finding: the note's other survivor — "'you may create' has no correct
  expression" — is defensible but weaker than stated. `Effect::MayPayThenEffect` establishes
  the corpus precedent of resolving a no-downside optional as pay/do-when-able; a free "you
  may create a 1/1" has no downside. Judgement call for whoever repairs it.
- **Comment fix only.**

### F8 — `wake_the_dead.rs`: inline TODO item 1 is stale and contradicted by the same file

- **File**: `crates/card-defs/src/defs/wake_the_dead.rs:26-34`
- **Stale**: TODO item 1 reads *"{X} in mana cost: ManaCost has no X field; … the mana cost
  itself can't represent {X}{B}{B} properly."* The def's own `mana_cost` now carries
  `x_count: 1` (line 15), the header comment at lines 16-19 explicitly records that the
  "not expressible" claim "was never true", and the `Completeness::inert` note says
  "`ManaCost.x_count` DOES exist ({X}{B}{B} is representable); that is no longer a blocker."
  Three statements in one file; one of them is wrong. Delete TODO item 1.
- **Items 2-4 re-verified as TRUE**, so `abilities: vec![]` is correct:
  (2) `TimingRestriction` (`card_definition.rs:4350`) has exactly two variants,
  `SorcerySpeed` and `AnyTime` — "cast only during combat on an opponent's turn" is
  inexpressible; (3) `TargetRequirement::UpToN` takes a fixed `count: u32`
  (`card_definition.rs:3022`), so "X target creature cards" has no expression;
  (4) delayed sacrifice exists only as `TokenSpec.delayed_action`
  (`card_definition.rs:2241`) and `Effect::ExileWithDelayedReturn`
  (`card_definition.rs:2084`), neither of which covers reanimated nontoken cards.
- `oracle_text` matches print exactly. **Comment fix only.**

### F9 — `whitemane_lion.rs`: non-targeted "a creature you control" modelled as a target

- **File**: `crates/card-defs/src/defs/whitemane_lion.rs:36-39`
- **Severity**: LOW despite the def being deck-legal `Complete`, because it is a
  **corpus-wide convention, not a CARDS-2 regression** — `shrieking_drake.rs:36-39` is
  byte-for-byte the same construction for identical printed text.
- **Printed**: "When this creature enters, return **a** creature you control to its owner's
  hand." — no "target". The choice is made on resolution (CR 608.2), not on announcement.
- **Def does**: `targets: vec![TargetRequirement::TargetCreatureWithFilter(TargetFilter {
  controller: TargetController::You, .. })]`.
- **Observable divergence**: the creature is locked in when the trigger goes on the stack
  rather than chosen at resolution; a creature you control with protection from white or
  "can't be the target of spells or abilities" is wrongly excluded from the choice; and the
  trigger becomes removable by making the chosen creature an illegal target. The trigger
  can never fizzle for lack of a choice (the Lion itself always qualifies), so no
  never-resolves failure mode.
- **Repair needs a primitive** — a non-targeted "choose a permanent you control" effect
  selection at resolution. Do **not** fix this def alone; it should be a corpus-wide sweep
  (start from `shrieking_drake`) or left as documented convention. Everything else on the
  def is correct: `{1}{W}`, `Creature — Cat`, 2/2, Flash, and `PlayerTarget::OwnerOf` is the
  right multiplayer answer for "its owner's hand".

### F10 — `brainsurge.rs`: header comment states the arithmetic backwards

- **File**: `crates/card-defs/src/defs/brainsurge.rs:5-7`
- **Stale/wrong**: header says *"Approximated as DrawCards(4) only (**net +2 is correct** but
  library ordering is wrong)."* That is false — `DrawCards(4)` with no put-back is net **+4**.
  The inline comment 19 lines below gets it right: *"net card advantage is wrong (+4 vs +2)."*
- The `Completeness::partial` marker is correct and keeps the def out of decks, so this is a
  documentation defect only. The "deferred to M10" framing is also stale phrasing (the
  PB-DP suite shipped blocking pending-decision machinery), but the substantive gap is real:
  I found **no** `Effect` variant for moving chosen cards from hand to the top of the library
  (searched `card_definition.rs` for `PutCardsFromHandOnTop`/`FromHandToLibrary`/
  `PutOnTopOfLibrary`/`ChooseCardsInHand` and for hand→`ZoneTarget::Library` shapes — the
  only hand→library effects are `ShuffleHandIntoLibrary` (`:2531`) and
  `ShuffleHandAndGraveyardIntoLibrary` (`:2534`)). **Comment fix only.**

### F11 — `windbrisk_heights.rs`: printed condition is on the effect, def makes it an activation gate

- **File**: `crates/card-defs/src/defs/windbrisk_heights.rs:61`
- **Printed**: "{W}, {T}: **You may play the exiled card without paying its mana cost if you
  attacked with three or more creatures this turn.**" — the attack condition qualifies the
  *effect*; the ability itself has no activation restriction, so it is legal to activate with
  zero attacks (and simply does nothing).
- **Def does**: `activation_condition: Some(Condition::YouAttackedWithNOrMore(3))`, which
  forbids activation outright below three attackers.
- **Impact is near-nil** — activating for no effect is never advantageous, and the def's
  version strictly prevents a wasted `{W}` and tap. Recorded for completeness, not for repair;
  changing it would trade a harmless restriction for a footgun. Everything else on the def is
  correct against print: Hideaway 4, enters tapped, `{T}: Add {W}`, `{W}, {T}` cost, and
  `oracle_text` matches exactly.

---

## Clean (11 of 22)

No finding of any severity. Every printed clause authored, nothing invented,
`oracle_text` byte-matching the MCP lookup, and every claimed blocker re-verified against
current source.

| Def | Why it is clean |
|---|---|
| `backup_agent` | `{1}{W}` Human Citizen 1/1; ETB `AddCounter(PlusOnePlusOne, 1)` on `TargetRequirement::TargetCreature`. Printed clause is genuinely "target creature", so the targeting model is right here (unlike F9). `Complete`, correctly. |
| `changeling_hero` | `{4}{W}` Shapeshifter 4/4; all three printed abilities present — `Changeling`, `Lifelink`, `Champion { filter: ChampionFilter::AnyCreature }` matching printed "Champion a creature". `oracle_text` matches print exactly. |
| `cyber_conversion` | `inert`, `abilities: vec![]`. Claimed gap re-verified: `card_definition.rs` contains **no** `FaceDown`/`TurnFaceDown`/`FaceDownKind` occurrence at all, so there is no primitive to turn an already-on-battlefield permanent face down in place. W5-correct. The note also correctly records the *previous* def's invented "add Artifact until end of turn + draw a card". |
| `exalted_angel` | `partial`, Flying + Morph `{2}{W}{W}` (cost verified) authored, damage trigger correctly omitted. Claimed gap re-verified: the only damage-direction triggers are `WhenDealsCombatDamageToPlayer` (`:3268`) and `WhenEnchantedCreatureDealsDamageToPlayer` (`:3513`); there is no general `WhenDealsDamage`. Removing the wrong static `Lifelink` was the right W5 call — the printed ability is triggered and Stifle-able. |
| `glistener_elf` | `{G}` Phyrexian Elf Warrior 1/1, `Infect`. Nothing else printed. `Complete`. |
| `lonely_sandbar` | Land; enters-tapped replacement, `{T}: Add {U}` (`mana_pool(0,1,0,0,0,0)` — WUBRGC order correct), Cycling marker + `AbilityDefinition::Cycling { cost: {U} }`. All three printed lines present, `oracle_text` exact. `Complete` via default and legitimately so. |
| `mindbreak_trap` | `inert`, `abilities: vec![]`, `Instant — Trap` subtype present, `oracle_text` exact. Both blockers real: no `AltCostKind::Trap` wrapper, and no variable-count targeting for "any number of target spells". The note correctly records that the prior single `TargetSpell` was wrong game state (forced exactly one target on the card that exists to answer storm turns). |
| `mystic_remora` | `known_wrong`, and the note is exactly right. Re-verified: `Effect::MayPayOrElse` at `crates/engine/src/effects/mod.rs:4147` still reads `execute_effect_inner(state, or_else, ctx, events)` unconditionally — the opponent is never offered the `{4}`. Cumulative upkeep `{1}` (both marker and `AbilityDefinition::CumulativeUpkeep`) and `noncreature_only: true` are correct; `oracle_text` exact. |
| `necron_deathmark` | `{3}{B}{B}` `Artifact Creature — Necron` 5/3; Flash + ETB `Sequence([DestroyPermanent(DeclaredTarget 0), MillCards(DeclaredTarget 1, 3)])` with `targets: [UpToN { count: 1, inner: TargetCreature }, TargetPlayer]`. "up to one target creature" and "target player" both modelled exactly. `oracle_text` exact including the "Synaptic Disintegrator —" ability word. `Complete` via default, correctly. |
| `overlord_of_the_hauntwoods` | `{3}{G}{G}` `Enchantment Creature — Avatar Horror` 6/5, not Legendary (correct). `Impending` marker + `AbilityDefinition::Impending { cost: {1}{G}{G}, count: 4 }` matching printed "Impending 4—{1}{G}{G}". "Enters or attacks" split into `WhenEntersBattlefield` + `WhenAttacks` — correct, that is two triggers by CR. Everywhere token: colorless, no supertypes (correctly **not** Basic), all five basic land subtypes, `tapped: true`. **The explicit `mana_abilities` are necessary, not a double** — I searched `crates/engine/src` and found no intrinsic-mana-ability derivation from basic land subtypes (only CR 305.6 *Domain counting* at `layers.rs:2408` / `effects/mod.rs:8405`), which is also why `lonely_sandbar` and `windbrisk_heights` author explicit `{T}: Add` lines. `oracle_text` exact. |
| `torment_of_hailfire` | `inert`, `abilities: vec![]`, `{X}{B}{B}` with `x_count: 1`, `oracle_text` exact. Blocker re-verified: the two-option "sacrifice a nonland permanent **or** discard a card" election needs interactivity `MayPayOrElse` does not have (same `effects/mod.rs:4147` site), plus an OR-cost variant. The note's positive half is also right — `Effect::Repeat` with `XValue` would cover "repeat X times". |

---

## Cross-cutting recommendations

1. **The dominant defect class in this batch is the stale in-file TODO, not the wrong
   field.** Seven of eleven findings (F1, F2, F6, F7, F8, F10, and the comment half of F3)
   are comments asserting a DSL gap that has closed, or asserting facts the same file
   contradicts elsewhere. Three files (`the_world_tree`, `prowess_of_the_fair`,
   `wake_the_dead`) contain an inline `// TODO` and a `Completeness` note that **disagree
   with each other**, with the note correct in all three cases. The mechanism is visible:
   when a repair pass rewrites the `Completeness` note it does not revisit the inline TODOs,
   and the inline TODO is what the *next* author reads first.
   Suggested rule: **a def has exactly one place where blockers are stated — the
   `Completeness` note.** Inline `// TODO` next to `abilities` should say nothing but "see
   completeness note".
2. **F1 and F2 are the only two findings that unblock coverage.** Both are card-def-only,
   both use primitives that already exist and already have corpus precedent, and F1 promotes
   `chord_of_calling` to `Complete` outright.
3. **F3 is the highest-value hygiene fix** even though it changes no behaviour: a fictional
   `oracle_text` on a `known_wrong` def is exactly the input that caused the earlier
   three-invented-abilities incident. Text-only repairs on blocked defs are cheap and remove
   the trap.
4. **F9 should not be fixed def-by-def.** It is a corpus-wide modelling convention for
   non-targeted "a creature you control" choices; `shrieking_drake` is identical. Either
   file it as a primitive seed (resolution-time choice of a permanent you control) or leave
   it documented, but do not repair one of the two.
