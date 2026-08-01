# PB-DX4 — class-B/class-D triage, batch 2 of 7

Date: 2026-08-01. Read-only triage. Oracle text from `mcp__mtg-rules__lookup_card`
(authoritative); defs read in full from `crates/card-defs/src/defs/`.

Note on MCP and multi-face cards: for `Consign // Oblivion` and
`Disciple of Freyalise // Garden of Freyalise` the MCP tool returns the combined name,
combined type line and (for Consign only) the combined mana cost, but **no per-face
oracle text**. Per-face clause comparison for those two is therefore against printed
text known outside MCP and is flagged WATCH rather than asserted.

Note on declared completeness: every def in this batch except `disciple_of_freyalise.rs`
closes with `..Default::default()`, i.e. it declares **no `completeness` field** and is
`Complete` by the `#[default]` derive on `Completeness` (`card_definition.rs:196-200`).
This is the `aurelia_the_warleader` / `emeria_the_sky_ruin` silent-marker class recorded
in CLAUDE.md. It is recorded per card below but is not by itself a class-D finding.

---

### Chaos Warp — **CLASS B**
- declared completeness: `Complete` (BY `#[default]` — no field declared)
- MCP printed oracle text: "The owner of target permanent shuffles it into their library, then reveals the top card of their library. If it's a permanent card, they put it onto the battlefield."
- stored `oracle_text` field: matches (verbatim)
- verdict rationale: mana cost `{2}{R}`, type `Instant`, and `TargetRequirement::TargetPermanent` all match. The def moves the target to its **owner's** library (`PlayerTarget::OwnerOf(DeclaredTarget{0})`, not `Controller` — correct in multiplayer), shuffles that owner's library, then `RevealAndRoute`s one card with `matched_dest: Battlefield` / `unmatched_dest: Library Top` — a nonpermanent card correctly stays revealed on top. No auto-choice deviation beyond the BASELINE premise.
- WATCH: the permanent-card filter (`has_card_types`, chaos_warp.rs:51-57) lists Artifact/Creature/Enchantment/Land/Planeswalker and omits Battle. `CardType::Battle` does **not exist** in the `CardType` enum (`rg 'Battle,' crates/card-types/src/cards/card_definition.rs` → no match), so this is not a def defect — there are no battle cards in the corpus to miss.

### Chart a Course — **CLASS B**
- declared completeness: `Complete` (BY `#[default]` — no field declared)
- MCP printed oracle text: "Draw two cards. Then discard a card unless you attacked this turn."
- stored `oracle_text` field: matches (verbatim)
- verdict rationale: `{1}{U}` Sorcery; `DrawCards(2)` then `Conditional { condition: YouAttackedThisTurn, if_true: Nothing, if_false: DiscardCards(1) }`. Polarity is right (discard on the *false* branch), the discard is mandatory as printed, and the sequence order matches. The only engine-side choice is *which* card is discarded — the BASELINE premise.

### Coiling Oracle — **CLASS B**
- declared completeness: `Complete` (BY `#[default]` — no field declared)
- MCP printed oracle text: "When this creature enters, reveal the top card of your library. If it's a land card, put it onto the battlefield. Otherwise, put that card into your hand."
- stored `oracle_text` field: DIFFERS trivially — "When **this** enters, reveal the top card of your library. …" (drops the word "creature" from the current WotC self-reference templating; the rest is verbatim)
- verdict rationale: `{G}{U}`, `Creature — Snake Elf Druid` (all three subtypes present, in printed order), 1/1, `WhenEntersBattlefield` → `RevealAndRoute` with `has_card_type: Land`, matched → `Battlefield { tapped: false }`, unmatched → `Hand { Controller }`. Every clause present, nothing extra. Land enters untapped as printed.
- WATCH: the one-word stored-`oracle_text` drift above. Cosmetic; not counted as class D (no behavioural clause is implicated and no trigger/duration/filter is misdescribed) — unlike Shambling Ghast, where the stored text named the wrong *trigger*.

### Consign // Oblivion — **CLASS B**
- declared completeness: `Complete` (BY `#[default]` — no field declared)
- MCP printed data: name "Consign // Oblivion", Mana Cost "{1}{U} // {4}{B}", Type "Instant // Sorcery", Keywords ["Aftermath"]. MCP returns **no per-face oracle text** for this split card.
- stored `oracle_text` field: "Return target nonland permanent to its owner's hand.\nOblivion — Aftermath (Cast this spell only from your graveyard. Then exile it.) Target player discards two cards." — consistent with the printed halves.
- verdict rationale: front half `{1}{U}` Instant with `TargetPermanentWithFilter(TargetFilter { non_land: true })` (KI-1 satisfied — not a bare `TargetPermanent`) bouncing to `PlayerTarget::OwnerOf(DeclaredTarget{0})`, i.e. *owner*, not controller — correct in multiplayer. Back half is a real `AbilityDefinition::Aftermath { cost: {4}{B}, card_type: Sorcery }` with `TargetPlayer` + `DiscardCards(2)` at `PlayerTarget::DeclaredTarget{0}` (the targeted player, not the caster). The `KeywordAbility::Aftermath` marker is present alongside the cost-bearing `Aftermath` definition (KI-6 satisfied). Both mana costs match MCP exactly.
- WATCH: per-face oracle wording unverifiable through MCP; comparison rests on printed text known outside the tool. Which two cards the targeted player discards is engine-chosen — BASELINE premise.

### Contagion Clasp — **CLASS B**
- declared completeness: `Complete` (BY `#[default]` — no field declared)
- MCP printed oracle text: "When this artifact enters, put a -1/-1 counter on target creature.\n{4}, {T}: Proliferate. (Choose any number of permanents and/or players, then give each another counter of each kind already there.)"
- stored `oracle_text` field: matches (verbatim, reminder text included)
- verdict rationale: `{2}` Artifact; ETB trigger with `TargetRequirement::TargetCreature` and `AddCounter { MinusOneMinusOne, count: 1 }` (permanent counter — correct here, the printed counter is a real counter, *unlike* Shambling Ghast's "until end of turn"); activated `Cost::Sequence([Mana{generic:4}, Tap])` → `Effect::Proliferate`. Nothing missing, nothing extra. Which permanents/players proliferate and which creature is targeted are engine-chosen — BASELINE premise.

### Contaminant Grafter — **CLASS D**
- declared completeness: `Complete` (BY `#[default]` — no field declared)
- MCP printed oracle text: "Trample, toxic 1\nWhenever one or more creatures you control deal combat damage to one or more players, proliferate.\nCorrupted — At the beginning of your end step, if an opponent has three or more poison counters, draw a card, then you may put a land card from your hand onto the battlefield."
- stored `oracle_text` field: matches (verbatim)
- verdict rationale: `{4}{G}`, `Creature — Phyrexian Druid` 5/5, `Trample` + `Toxic(1)` keywords, the batch combat-damage trigger, and the Corrupted intervening-if (`Condition::OpponentHasPoisonCounters(3)`, correctly gating at trigger time) are all faithful. **But the printed optionality on the land-put is dropped**, which makes the def do something the card cannot make you do.
- DEFECT 1: printed "draw a card, then **you may** put a land card from your hand onto the battlefield" vs def `Effect::Sequence(vec![DrawCards, Effect::PutLandFromHandOntoBattlefield { tapped: false }])` at `crates/card-defs/src/defs/contaminant_grafter.rs:50-56` (the land-put is line 55). The effect has **no optional wrapper of any kind** — no `MayPayThenEffect`, no `Conditional` — and the engine handler is unconditional: `crates/engine/src/effects/mod.rs:6640-6646` picks "the land card with the lowest ObjectId" and puts it onto the battlefield, with the DSL doc at `card_definition.rs:2350-2359` confirming the only escape is "if no land card is in hand, the effect does nothing". **Effect on play:** every end step at which an opponent has 3+ poison, the controller is *forced* to empty a land out of hand onto the battlefield — losing the choice to hold it for a discard outlet, to avoid feeding an opponent's landfall/permanent-count punisher, or to keep hand information hidden. This is the Smuggler's Copter shape exactly (printed "may" authored as a mandatory `Sequence` on a `Complete` def).
- Note on the correct marker: a *free* "you may" has no expressible form in this DSL (`MayPayThenEffect` requires a `Cost` and a free cost always trivially pays; `Effect::Choose` and `MayPayOrElse` are barred from `Complete` by `effect_choose_gate.rs`) — the same class as `emeria_the_sky_ruin`'s printed "you **may** return", which PB-DX3b marked **explicit `partial`**. That precedent is the disposition this def should get, not a rewrite.
- WATCH (separate, class-B): *which* land is put onto the battlefield is engine-auto-chosen (lowest ObjectId). That half is the BASELINE premise and is not part of the class-D claim.

### Contentious Plan — **CLASS B**
- declared completeness: `Complete` (BY `#[default]` — no field declared)
- MCP printed oracle text: "Proliferate. (Choose any number of permanents and/or players, then give each another counter of each kind already there.)\nDraw a card."
- stored `oracle_text` field: matches in substance — "Proliferate.\nDraw a card." (reminder text omitted; reminder text is not rules text)
- verdict rationale: `{1}{U}` Sorcery, `Sequence([Proliferate, DrawCards(1)])` in printed order, drawn by `PlayerTarget::Controller`. Which permanents/players are proliferated is engine-chosen — BASELINE premise.

### Crippling Fear — **CLASS B**
- declared completeness: `Complete` (BY `#[default]` — no field declared)
- MCP printed oracle text: "Choose a creature type. Creatures that aren't of the chosen type get -3/-3 until end of turn."
- stored `oracle_text` field: matches (verbatim)
- verdict rationale: `{2}{B}{B}` Sorcery; `ChooseCreatureType` then `ApplyContinuousEffect` at `EffectLayer::PtModify` with `ModifyBoth(-3)`, `EffectFilter::AllCreaturesExcludingChosenSubtype`, `duration: UntilEndOfTurn`. Amount, layer, **duration** and the "all creatures, not just opponents'" scope are all correct. The `default: SubType("Human")` on `ChooseCreatureType` is the engine auto-choosing the type — the BASELINE premise, and the reason this def is in the table.

### Crossway Troublemakers — **CLASS B**
- declared completeness: `Complete` (BY `#[default]` — no field declared)
- MCP printed oracle text: "Attacking Vampires you control have deathtouch and lifelink. (…)\nWhenever a Vampire you control dies, you may pay 2 life. If you do, draw a card."
- stored `oracle_text` field: matches (verbatim, reminder text omitted)
- verdict rationale: `{5}{B}` `Creature — Vampire` 5/5. Both grants use `EffectFilter::AttackingCreaturesYouControlWithSubtype(Vampire)` — the "attacking" and "you control" restrictions are both present, and deathtouch/lifelink are granted as two separate `Ability`-layer statics with `WhileSourceOnBattlefield`. The death trigger is `WheneverCreatureDies { controller: Some(You), filter: has_subtype Vampire }` with `exclude_self: false` — correct, because Crossway Troublemakers is itself a Vampire you control and its own death does trigger it (CR 603.6d leaves-the-battlefield). The printed "you may pay 2 life. If you do, draw a card" is faithfully `Effect::MayPayThenEffect { cost: PayLife(2), then: DrawCards(1) }` — optionality **preserved**, in contrast to Contaminant Grafter above.

### Deflecting Swat — **CLASS B**
- declared completeness: `Complete` (BY `#[default]` — no field declared)
- MCP printed oracle text: "If you control a commander, you may cast this spell without paying its mana cost.\nYou may choose new targets for target spell or ability."
- stored `oracle_text` field: matches (verbatim)
- verdict rationale: `{2}{R}` Instant; `AltCastAbility { kind: AltCostKind::CommanderFreeCast }` for the free-cast clause and `Effect::ChangeTargets { must_change: false }` for the "**may** choose new targets" — `must_change: false` is the correct encoding of the optionality. The engine then deterministically declines to change anything (comment, deflecting_swat.rs:38: "targets left unchanged (player 'chose' not to change)"), which makes the card an on-resolution no-op today — but that *is* the auto-choice the BASELINE records, so it is class B by the premise, not a defect of the def.
- WATCH: printed says "target spell **or ability**"; the def declares `TargetRequirement::TargetSpell` (deflecting_swat.rs:39), whose doc comment reads `/// "target spell"` (`card_definition.rs:2955-2956`). In practice this is **not** currently a narrowing: the validator for `TargetSpell` checks only `obj.zone != ZoneId::Stack` (`crates/engine/src/rules/casting.rs:6303-6313`) and never checks `StackObjectKind`, so activated/triggered abilities on the stack are legal targets today. The def therefore behaves as printed — but only because of an engine looseness, and it would silently become narrower-than-printed the moment `TargetSpell` is tightened to spells (the tight variants `TargetSpellWithSingleTarget` / `TargetSpellOrAbilityWithSingleTarget` already exist and do check kind). Flagged, not classed D.

### Demon's Disciple — **CLASS B**
- declared completeness: `Complete` (BY `#[default]` — no field declared)
- MCP printed oracle text: "When this creature enters, each player sacrifices a creature or planeswalker of their choice."
- stored `oracle_text` field: matches (verbatim)
- verdict rationale: `{2}{B}` `Creature — Human Cleric` 3/1; `WhenEntersBattlefield` → `SacrificePermanents { player: PlayerTarget::EachPlayer, count: 1, filter: has_card_types [Creature, Planeswalker] }`. **Each player**, not each opponent — matches printed (the controller sacrifices too, and Demon's Disciple itself is a legal sacrifice). The OR-type filter is right. "of their choice" being resolved by the engine is the BASELINE premise.

### Dictate of Erebos — **CLASS B**
- declared completeness: `Complete` (BY `#[default]` — no field declared)
- MCP printed oracle text: "Flash\nWhenever a creature you control dies, each opponent sacrifices a creature of their choice."
- stored `oracle_text` field: matches (verbatim)
- verdict rationale: `{3}{B}{B}` Enchantment with `KeywordAbility::Flash` present. Trigger is `WheneverCreatureDies { controller: Some(TargetController::You) }` — correctly scoped to creatures **you** control, not all deaths (KI-9 satisfied); `exclude_self` is irrelevant because the source is an enchantment. The effect is `ForEach { over: ForEachTarget::EachOpponent, … SacrificePermanents { player: DeclaredTarget{0}, count: 1, filter: Creature } }` — **each opponent**, not each player, matching printed, and each opponent sacrifices their *own* creature (`DeclaredTarget{0}` inside `ForEach::EachOpponent` is the per-iteration player, the documented-correct idiom). Which creature each opponent loses is engine-chosen — BASELINE premise.

### Disciple of Freyalise // Garden of Freyalise — **CLASS B**
- declared completeness: `Complete` — **explicitly declared** (`completeness: Completeness::Complete`, disciple_of_freyalise.rs:107); the only def in this batch that declares the field
- MCP printed data: name "Disciple of Freyalise // Garden of Freyalise", Type "Creature — Elf Druid // Land", Color identity ["G"]. MCP returns **no per-face oracle text, no mana cost and no P/T** for this card. Rulings confirm it is a **modal** DFC ("A modal double-faced card can't be transformed…") and confirm the X clause: "Use the power of the sacrificed creature as it last existed on the battlefield to determine the value of X."
- stored `oracle_text` field: front "When this creature enters, you may sacrifice another creature. If you do, you gain X life and draw X cards, where X is that creature's power."; back "As this land enters, you may pay 3 life. If you don't, it enters tapped.\n{T}: Add {G}." — consistent with the printed faces.
- verdict rationale: front `Creature — Elf Druid` 3/3; the ETB is `MayPayThenEffect { cost: Cost::Sacrifice(TargetFilter { has_card_type: Creature, exclude_self: true }) }` — the printed "**you may**" is preserved as a real optional cost, and "**another** creature" is enforced by `exclude_self: true` (CR 109.1). Both payoffs use `EffectAmount::PowerOfSacrificedCreature`, matching the LKI ruling MCP returned verbatim. Back face is `types(&[CardType::Land])` with a `ReplacementModification::EntersTappedUnlessPayLife(3)` self-replacement (matching printed "you may pay 3 life. If you don't, it enters tapped" — this is a genuine ETB-tapped condition, so the replacement is required, KI-13/14 satisfied) and `{T}: Add {G}` via `mana_pool(0,0,0,0,1,0)` — WUBRGC order, green = 1. Correct.
- WATCH 1: front-face mana cost `{3}{G}{G}{G}` and P/T 3/3 could **not** be verified against MCP (the tool returns no per-face cost or P/T for this card). Unresolved, not asserted.
- WATCH 2: this is an MDFC, and the back face is expressed with the generic `back_face: Some(CardFace { … })`, the same slot transforming DFCs use — the DSL has no MDFC marker at all (`rg -i 'MDFC|modal_dfc|is_mdfc' crates/card-types/src` → no match) and `Command::PlayLand` carries only a card id, no face selector. Whether "Garden of Freyalise" is actually *playable* as a land is therefore an open engine question. Recorded as B because authoring an MDFC back face into `back_face` is the corpus-wide convention here (`bala_ged_recovery.rs`, `sea_gate_restoration.rs`, `sundering_eruption.rs`) and defs that *omit* the back face are marked `partial` (`valakut_awakening.rs`, `sejiri_shelter.rs`, `boggart_trawler.rs`) — i.e. this def follows the accepted pattern. Any gap here is engine-wide, not this def's.

### Dreadhorde Invasion — **CLASS B**
- declared completeness: `Complete` (BY `#[default]` — no field declared)
- MCP printed oracle text: "At the beginning of your upkeep, you lose 1 life and amass Zombies 1. (…)\nWhenever a Zombie token you control with power 6 or greater attacks, it gains lifelink until end of turn."
- stored `oracle_text` field: matches (verbatim, reminder text included)
- verdict rationale: `{1}{B}` Enchantment. Upkeep trigger is `AtBeginningOfYourUpkeep` → `Sequence([LoseLife(Controller, 1), Amass { subtype: "Zombie", count: 1 }])` — **lose** life (not pay, not damage), amount 1, amass 1, in printed order, and the loss is mandatory as printed. Attack trigger is `WheneverCreatureYouControlAttacks` with `filter: { has_subtype: Zombie, min_power: Some(6), is_token: true }` — all three printed restrictions (Zombie / token / power 6 or greater) present, and "you control" comes from the trigger variant. The grant is `AddKeyword(Lifelink)` at `EffectLayer::Ability` aimed by `EffectFilter::TriggeringCreature` — i.e. at **it**, the attacking Zombie, not all creatures — with `duration: UntilEndOfTurn` matching printed "until end of turn". No auto-choice deviation beyond the BASELINE premise (which Army receives the amass counter).

---

SUMMARY batch2: 13 class-B, 1 class-D (Contaminant Grafter), 6 watch
