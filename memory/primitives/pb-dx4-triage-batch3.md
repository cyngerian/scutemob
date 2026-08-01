# PB-DX4 — class-B/class-D triage, batch 3 of 7

**Date**: 2026-08-01
**Cards**: 14 (Dromoka → Goblin Ringleader, `BASELINE` alphabetical slice)
**Source of truth**: `mcp__mtg-rules__lookup_card` printed oracle text. The def's own stored
`oracle_text` field was checked *against* MCP, never trusted.
**Read-only**: no file was edited.

Recorded `BASELINE` rows for this slice (`crates/engine/tests/core/decision_gate.rs:399-412`),
used to establish what each def's *expected* auto-choice is:

```
399  ("Dromoka, the Eternal",      &["bolster_amass"],         None),
400  ("Drown in Ichor",            &["proliferate"],           None),
401  ("Etchings of the Chosen",    &["choose_color_or_type"],  None),
402  ("Evolution Sage",            &["proliferate"],           None),
403  ("Faithless Looting",         &["discard_cards"],         None),
404  ("Felidar Retreat",           &["modal_trigger"],         None),
405  ("Fell Specter",              &["discard_cards"],         None),
406  ("Fleshbag Marauder",         &["sacrifice_permanents"],  None),
407  ("Flusterstorm",              &["counter_unless_pays"],   None),
408  ("Flux Channeler",            &["proliferate"],           None),
409  ("Frantic Search",            &["discard_cards"],         None),
410  ("Geier Reach Sanitarium",    &["discard_cards"],         None),
411  ("Geological Appraiser",      &["discover"],              None),
412  ("Goblin Ringleader",         &["look_at_top_or_route"],  None),
```

Every one of these rows is a genuine engine-side auto-choice (which permanent to bolster on a
toughness tie, which cards to proliferate, which card to discard, which creature to sacrifice,
which mode, whether to pay, what to do with the discovered card, bottom order). That is the
premise of the BASELINE and is **not** counted as a defect below.

---

### Dromoka, the Eternal — **CLASS B**
- declared completeness: `Complete` (BY `#[default]` — `..Default::default()` at
  `dromoka_the_eternal.rs:42`, no `completeness` field)
- MCP printed oracle text: "Flying\nWhenever a Dragon you control attacks, bolster 2. (Choose a
  creature with the least toughness among creatures you control and put two +1/+1 counters on it.)"
- stored `oracle_text` field: matches (reminder text omitted, which is conventional in this corpus)
  — "Flying\nWhenever a Dragon you control attacks, bolster 2."
- verdict rationale: `{3}{G}{W}` / `Legendary Creature — Dragon` / 5/5 all correct, `SuperType::Legendary`
  present. `WheneverCreatureYouControlAttacks` with `has_subtype: Some(Dragon)` matches "a Dragon you
  control" and correctly includes Dromoka herself (no `exclude_self`). `Effect::Bolster { player:
  Controller, count: Fixed(2) }` is bolster 2. The only auto-choice is the tie-break among equal-lowest
  toughness, which is the recorded `bolster_amass` row.

### Drown in Ichor — **CLASS B**
- declared completeness: `Complete` (BY `#[default]` — `drown_in_ichor.rs:36`)
- MCP printed oracle text: "Target creature gets -4/-4 until end of turn. Proliferate. (Choose any
  number of permanents and/or players, then give each another counter of each kind already there.)"
- stored `oracle_text` field: matches verbatim, reminder text included
- verdict rationale: `{1}{B}` Sorcery correct. `ModifyBoth(-4)` on `EffectLayer::PtModify` with
  `EffectDuration::UntilEndOfTurn` against `DeclaredTarget { index: 0 }`, sequenced before
  `Effect::Proliferate`, and `targets: vec![TargetRequirement::TargetCreature]` — exactly the printed
  clause order. Proliferate's "any number of permanents and/or players" is the recorded auto-choice.

### Etchings of the Chosen — **CLASS B**
- declared completeness: `Complete` (BY `#[default]` — `etchings_of_the_chosen.rs:79`)
- MCP printed oracle text: "As this enchantment enters, choose a creature type.\nCreatures you control
  of the chosen type get +1/+1.\n{1}, Sacrifice a creature of the chosen type: Target creature you
  control gains indestructible until end of turn. (Damage and effects that say \"destroy\" don't
  destroy it.)"
- stored `oracle_text` field: matches (reminder text omitted)
- verdict rationale: all three printed clauses present and correctly shaped. The "as it enters" choice
  is a `ReplacementModification::ChooseCreatureType(SubType("Human"))` — a hardcoded default — but
  `choose_color_or_type` is precisely this def's recorded `BASELINE` row, so the hardcoded type IS the
  auto-choice under triage, not an extra defect. Static is `EffectFilter::CreaturesYouControlOfChosenType`
  + `ModifyBoth(1)` on `PtModify`, `WhileSourceOnBattlefield` — correct. Activated cost is
  `Cost::Sequence([Mana{generic:1}, Sacrifice(TargetFilter{ has_card_type: Creature, has_chosen_subtype:
  true })])` — matches "{1}, Sacrifice a creature of the chosen type" including the chosen-type filter;
  target is `TargetCreatureWithFilter(controller: You)` matching "target creature you control";
  `AddKeyword(Indestructible)` on `EffectLayer::Ability`, `UntilEndOfTurn`.

### Evolution Sage — **CLASS B**
- declared completeness: `Complete` (BY `#[default]` — `evolution_sage.rs:40`)
- MCP printed oracle text: "Landfall — Whenever a land you control enters, proliferate. (Choose any
  number of permanents and/or players, then give each another counter of each kind already there.)"
- stored `oracle_text` field: matches verbatim, reminder text included
- verdict rationale: `{2}{G}` / `Creature — Elf Druid` / 3/2 correct. Trigger is
  `WheneverPermanentEntersBattlefield { filter: has_card_type Land + controller You }` — matches
  "a land you control enters". `exclude_self: false` is harmless here (an Elf Druid can never satisfy
  the land filter). `Effect::Proliferate` is the whole printed effect.

### Faithless Looting — **CLASS B**
- declared completeness: `Complete` (BY `#[default]` — `faithless_looting.rs:53`)
- MCP printed oracle text: "Draw two cards, then discard two cards.\nFlashback {2}{R} (You may cast
  this card from your graveyard for its flashback cost. Then exile it.)"
- stored `oracle_text` field: matches verbatim, reminder text included
- verdict rationale: `{R}` Sorcery correct. Effect is `Sequence([DrawCards{Controller, 2},
  DiscardCards{Controller, 2}])` — and this is **mandatory in print** ("Draw two cards, then discard
  two cards" — no "may"), so unlike Smuggler's Copter there is no dropped optionality. Flashback is
  correctly dual-defined (KI-6): both `KeywordAbility::Flashback` and `AltCastAbility { kind:
  Flashback, cost: {2}{R} }`. Which two cards get discarded is the recorded `discard_cards` row.

### Felidar Retreat — **CLASS B** (with WATCH)
- declared completeness: `Complete` (BY `#[default]` — `felidar_retreat.rs:102`)
- MCP printed oracle text: "Landfall — Whenever a land you control enters, choose one —\n• Create a
  2/2 white Cat Beast creature token.\n• Put a +1/+1 counter on each creature you control. Those
  creatures gain vigilance until end of turn."
- stored `oracle_text` field: matches verbatim
- verdict rationale: `{3}{W}` Enchantment correct. Landfall trigger filter correct. `ModeSelection {
  min_modes: 1, max_modes: 1, allow_duplicate_modes: false }` is "choose one". Mode 0 token is 2/2,
  white, `Creature — Cat Beast`, count 1 — correct. Mode 1's counter half is `ForEach {
  EachCreatureYouControl, AddCounter{ DeclaredTarget{0}, PlusOnePlusOne, 1 } }` — the
  `DeclaredTarget { index: 0 }`-inside-`ForEach` idiom is the established correct pattern in this
  corpus and is not a bug. Which mode gets picked is the recorded `modal_trigger` row.
- **WATCH**: printed says "**Those** creatures gain vigilance until end of turn" — i.e. a fixed set
  captured on resolution. The def authors it as `EffectFilter::CreaturesYouControl` with
  `EffectDuration::UntilEndOfTurn` (`felidar_retreat.rs:82-92`). Because `ContinuousEffect` has no
  affected-set field and `layers.rs` re-evaluates the filter live, a creature that enters *later* the
  same turn would also gain vigilance. This is exactly the filed **OOS-OS7-2 / CR 611.2c** class
  (7 `Complete` defs, ranked as PB-DX5) — an engine affected-set gap, not a defect unique to this def,
  and the DSL has no way to express it correctly today. Held at B rather than promoted to D so the
  count is not inflated with an already-tracked engine seed; flag it if PB-DX5's roster is being built.

### Fell Specter — **CLASS B**
- declared completeness: `Complete` (BY `#[default]` — `fell_specter.rs:54`)
- MCP printed oracle text: "Flying\nWhen this creature enters, target opponent discards a
  card.\nWhenever an opponent discards a card, that player loses 2 life."
- stored `oracle_text` field: matches verbatim
- verdict rationale: `{3}{B}` / `Creature — Specter` / 1/3 correct. ETB uses
  `TargetRequirement::TargetOpponent` (not `TargetPlayer`) so the controller cannot self-target —
  the CR 601.2c legal-but-wrong hazard is already closed here by PB-EF6. Second trigger is
  `WheneverOpponentDiscards` → `LoseLife { player: PlayerTarget::TriggeringPlayer, amount: 2 }`,
  which is "**that player** loses 2 life", not the controller — the multiplayer recipient check
  passes. Which card the opponent discards is the recorded `discard_cards` row.

### Fleshbag Marauder — **CLASS B** (with WATCH)
- declared completeness: `Complete` (BY `#[default]` — `fleshbag_marauder.rs:38`)
- MCP printed oracle text: "When this creature enters, each player sacrifices a creature of their choice."
- stored `oracle_text` field: **DIFFERS** — "When this enters, each player sacrifices a creature."
- verdict rationale: `{2}{B}` / `Creature — Zombie Warrior` / 3/1 correct. `SacrificePermanents {
  player: PlayerTarget::EachPlayer, count: Fixed(1), filter: has_card_type Creature }` is "each player
  sacrifices a creature" — `EachPlayer`, correctly, not `EachOpponent`; the controller sacrifices too.
  "of their choice" is precisely the recorded `sacrifice_permanents` auto-choice row.
- **WATCH**: the stored `oracle_text` drift is two-fold — "this" for "this creature" (cosmetic
  retemplating, endemic in the corpus) and a dropped "of their choice" (`fleshbag_marauder.rs:15`).
  Neither changes encoded behaviour, and the dropped clause is the BASELINE row itself, so this is
  *not* the Shambling Ghast shape (where the stored text named a different trigger event). Cosmetic;
  worth a text refresh sweep, not a class-D demotion.

### Flusterstorm — **CLASS B**
- declared completeness: `Complete` (BY `#[default]` — `flusterstorm.rs:38`)
- MCP printed oracle text: "Counter target instant or sorcery spell unless its controller pays
  {1}.\nStorm (When you cast this spell, copy it for each spell cast before it this turn. You may
  choose new targets for the copies.)"
- stored `oracle_text` field: matches verbatim, reminder text included
- verdict rationale: `{U}` Instant correct. `Effect::CounterUnlessPays { target: DeclaredTarget{0},
  cost: Mana{generic:1} }` with `TargetSpellWithFilter(has_card_types: [Instant, Sorcery])` matches
  "target instant or sorcery spell" and the "{1}" is generic-1, not colored. Critically the payer is
  the spell's **controller** (that is what `CounterUnlessPays` means), not Flusterstorm's controller —
  the multiplayer "its controller" check passes. `KeywordAbility::Storm` present. Whether the
  opponent pays is the recorded `counter_unless_pays` row (PB-AC2 / CR 118.12a auto-decline).

### Flux Channeler — **CLASS B**
- declared completeness: `Complete` (BY `#[default]` — `flux_channeler.rs:39`)
- MCP printed oracle text: "Whenever you cast a noncreature spell, proliferate. (Choose any number of
  permanents and/or players, then give each another counter of each kind already there.)"
- stored `oracle_text` field: matches verbatim, reminder text included
- verdict rationale: `{2}{U}` / `Creature — Human Wizard` / 2/2 correct. `WheneverYouCastSpell {
  noncreature_only: true, during_opponent_turn: false, .. }` is "whenever **you** cast a noncreature
  spell" — correctly scoped to the controller, not any player. Effect is bare `Effect::Proliferate`.

### Frantic Search — **CLASS B** (with WATCH)
- declared completeness: `Complete` (BY `#[default]` — `frantic_search.rs:45`)
- MCP printed oracle text: "Draw two cards, then discard two cards. Untap up to three lands."
- stored `oracle_text` field: matches verbatim
- verdict rationale: `{2}{U}` Instant correct. Draw 2 / discard 2 is mandatory in print, so no dropped
  optionality. "Up to three lands" is `TargetRequirement::UpToN { count: 3, inner: TargetLand }` with
  three `UntapPermanent` effects on indices 0/1/2 — the "up to" is honoured (0 declared lands is legal
  and the extra slots no-op), and `TargetLand` carries no controller filter, correctly matching the
  printed text's silence about whose lands they are. Discard selection is the recorded `discard_cards` row.
- **WATCH**: printed says "Untap up to three lands" with **no** "target". The def makes them true
  targets (`frantic_search.rs:38-41`). Two consequences the printed card does not have: the lands are
  locked in at cast time rather than chosen on resolution, and — the sharper one — CR 608.2b means
  that if every declared land becomes an illegal target in response, the spell does not resolve **at
  all**, so the draw-2/discard-2 is lost. Printed Frantic Search can never fizzle. Held at B because
  this is a systemic DSL approximation (there is no non-targeted "choose up to N permanents" primitive)
  almost certainly shared by many defs, so it belongs in a corpus-wide sweep rather than a per-card D.

### Geier Reach Sanitarium — **CLASS B** (with strong WATCH — closest thing to a D in this batch)
- declared completeness: `Complete` (BY `#[default]` — `geier_reach_sanitarium.rs:56`)
- MCP printed oracle text: "{T}: Add {C}.\n{2}, {T}: Each player draws a card, then discards a card."
- stored `oracle_text` field: matches verbatim
- verdict rationale: `Legendary Land`, and `SuperType::Legendary` **is** present at
  `geier_reach_sanitarium.rs:9` (KI-4 clear — this is the exact trap `emeria_the_sky_ruin` fell into
  in PB-DX3b, in the opposite direction). `mana_cost: None` correct for a land. First ability
  `Cost::Tap` → `mana_pool(0,0,0,0,0,1)` = one colorless in WUBRGC order — correct, not a mis-ordered
  white. No ETB-tapped clause printed and none in the def (KI-13/14 clear). Second ability's cost
  `Sequence([Mana{generic:2}, Tap])` matches "{2}, {T}". Each player draws exactly 1 and discards
  exactly 1; `ForEachTarget::EachPlayer`, correctly not `EachOpponent` — the activating player is
  included. Which card each player pitches is the recorded `discard_cards` row.
- **WATCH**: the def executes draw-then-discard **per player, interleaved**:
  `Effect::ForEach { over: EachPlayer, effect: Sequence([DrawCards, DiscardCards]) }`
  (`geier_reach_sanitarium.rs:35-47`). The printed clause is "Each player draws a card, **then**
  discards a card", and the official ruling [2016-07-13] reads verbatim: *"first each player draws a
  card. Then the player whose turn it is selects a card from their hand and sets it aside without
  revealing it; proceeding in turn order, each other player does the same. Then the cards that were
  set aside are discarded at once."* So the printed card is all-draw → all-discard-simultaneously,
  while the def is P1 draws, P1 discards, P2 draws, P2 discards, … Effect on play: discard-triggered
  abilities (Waste Not, Fell Specter's own second trigger, madness) resolve interleaved with later
  players' draws instead of firing off one simultaneous batch, and a later player's draw happens with
  earlier players' discards already in the graveyard. This is a real, def-level, fixable deviation
  (two sequential `ForEach` loops would express it), and it has a verbatim printed clause plus an
  exact contradicting expression — but it is an ordering/simultaneity error, not a wrong count,
  duration, filter or dropped optionality, and its observable footprint is confined to trigger
  ordering while the discards remain engine-chosen anyway. Held at B per the "if unsure, WATCH"
  discipline; **this is the one entry in batch 3 a reviewer might reasonably promote to D**, and all
  the evidence needed to do so is recorded here.

### Geological Appraiser — **CLASS B**
- declared completeness: `Complete` (**EXPLICIT** — `completeness: Completeness::Complete` at
  `geological_appraiser.rs:47`; this def spells out every field rather than using
  `..Default::default()`, and is the only explicit declaration in batch 3)
- MCP printed oracle text: "When this creature enters, if you cast it, discover 3. (Exile cards from
  the top of your library until you exile a nonland card with mana value 3 or less. Cast it without
  paying its mana cost or put it into your hand. Put the rest on the bottom in a random order.)"
- stored `oracle_text` field: matches verbatim, reminder text included
- verdict rationale: `{2}{R}{R}` / `Creature — Human Artificer` / 3/2 correct. (MCP also returns
  `A-Geological Appraiser` at `{3}{R}{R}` — that is the Alchemy rebalance, not the paper card, and
  the def correctly encodes the paper cost.) The printed "**if you cast it**" is authored as
  `intervening_if: Some(Condition::WasCast)` (CR 603.4), so the trigger does not fire on a blink or a
  reanimation — the clause is present, not dropped. `Effect::Discover { player: Controller, n: 3 }`.
  The cast-vs-hand election on the discovered card is the recorded `discover` auto-choice row.

### Goblin Ringleader — **CLASS B**
- declared completeness: `Complete` (BY `#[default]` — `goblin_ringleader.rs:52`)
- MCP printed oracle text: "Haste (This creature can attack and {T} as soon as it comes under your
  control.)\nWhen this creature enters, reveal the top four cards of your library. Put all Goblin
  cards revealed this way into your hand and the rest on the bottom of your library in any order."
- stored `oracle_text` field: matches verbatim, reminder text included
- verdict rationale: `{3}{R}` / `Creature — Goblin` / 2/2 correct. `KeywordAbility::Haste` present.
  `Effect::RevealAndRoute { player: Controller, count: Fixed(4), filter: has_subtype Goblin,
  matched_dest: Hand{Controller}, unmatched_dest: Library{Controller, Bottom} }` maps clause for
  clause: four cards, **all** Goblins to hand (not "up to one"), remainder to the **bottom**
  (PB-RS1's top/bottom inversion class does not apply — `LibraryPosition::Bottom` is explicit). The
  "in any order" bottom ordering is the recorded `look_at_top_or_route` auto-choice row.

---

SUMMARY batch3: 14 class-B, 0 class-D, 4 watch (Felidar Retreat — "those creatures" vigilance
authored as a live-evaluated `CreaturesYouControl` filter, the already-filed OOS-OS7-2 / CR 611.2c
class; Fleshbag Marauder — stored `oracle_text` drift, cosmetic; Frantic Search — untargeted
"up to three lands" authored as real targets, so the spell can fizzle where the printed card cannot,
a systemic DSL approximation; Geier Reach Sanitarium — per-player interleaved draw/discard against a
printed "then" that the official ruling says is all-draw-then-simultaneous-discard, the strongest
promotion candidate in this batch).
