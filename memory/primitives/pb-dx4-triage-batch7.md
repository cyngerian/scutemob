# PB-DX4 — class-B / class-D triage, batch 7 of 7

**Date**: 2026-08-01
**Scope**: 13 `BASELINE` defs (PB-DP10 `crates/engine/tests/core/decision_gate.rs`).
**Method**: MCP `lookup_card` (printed oracle text, type line, mana cost, P/T, keywords) read
first and treated as authoritative; def file read in full; clause-by-clause comparison.
Read-only — no file was edited.

**Corpus-wide observation for this batch**: **all 13 defs declare NO `completeness` field.**
Every one ends `..Default::default()`, so every one is `Complete` by the `#[default]` derive —
the exact silent-defect generator that produced `aurelia_the_warleader` (PB-DX1) and
`emeria_the_sky_ruin` (PB-DX3b). 13/13 in this batch. That is a data point for the standing
"which defs never declare a marker at all?" question CLAUDE.md raises.

**Systemic (not per-card) note — Equip cost is unencoded.** `KeywordAbility::Equip` is a bare
marker taking no cost argument (`crates/card-types/src/state/types.rs:432`), and all **21**
equipment defs in the corpus use it that way. Both swords below print `Equip {2}` and neither
encodes the `{2}`. This is a DSL shape shared corpus-wide, not an authoring error in these two
defs, so it is recorded as a WATCH on each rather than as a class-D defect. It should be raised
as its own seed, not resolved per-card.

---

### Sword of Feast and Famine — **CLASS B**
- declared completeness: `Complete` (BY `#[default]` — no field declared)
- MCP printed oracle text: "Equipped creature gets +2/+2 and has protection from black and from green.\nWhenever equipped creature deals combat damage to a player, that player discards a card and you untap all lands you control.\nEquip {2}"
- stored `oracle_text` field: **matches** (verbatim, including `Equip {2}`)
- verdict rationale: Mana cost `{3}`, type line `Artifact — Equipment`, and all three printed
  clauses are present and faithful: `ModifyBoth(2)` on `AttachedCreature`, two separate
  `ProtectionFrom(FromColor(Black/Green))` statics, and a
  `WhenEquippedCreatureDealsCombatDamageToPlayer` trigger whose `Sequence` discards **1** from
  `PlayerTarget::DamagedPlayer` (correct player — not the controller) and then untaps every land
  matching `has_card_type: Land, controller: You`. The only engine-side auto-choice is *which*
  card the damaged player discards, which is precisely the BASELINE premise.
- WATCH: `Equip {2}` cost unencoded (systemic, see header). Also, the discard is authored as
  going to `DamagedPlayer` — correct — but note the untap is scoped `TargetController::You`
  relative to the *ability's* controller, which is right for "you untap all lands you control".

### Sword of Truth and Justice — **CLASS D**
- declared completeness: `Complete` (BY `#[default]` — no field declared)
- MCP printed oracle text: "Equipped creature gets +2/+2 and has protection from white and from blue.\nWhenever equipped creature deals combat damage to a player, put a +1/+1 counter on a creature you control, then proliferate. (Choose any number of permanents and/or players, then give each another counter of each kind already there.)\nEquip {2}"
- stored `oracle_text` field: **DIFFERS (cosmetic only)** — "Equipped creature gets +2/+2 and has
  protection from white and from blue.\nWhenever equipped creature deals combat damage to a
  player, put a +1/+1 counter on a creature you control, then proliferate.\nEquip {2}"
  (the parenthetical proliferate reminder text is omitted; wording of the rules text itself matches)
- verdict rationale: The +2/+2, both protections and the `Proliferate` half are faithful. The
  defect is the **filter on the +1/+1 counter recipient**: the printed clause restricts it to a
  creature **you control**, and the def encodes an unrestricted `TargetRequirement::TargetCreature`.
- **DEFECT 1**: printed "put a +1/+1 counter on **a creature you control**, then proliferate" vs
  def `targets: vec![TargetRequirement::TargetCreature]` at
  `crates/card-defs/src/defs/sword_of_truth_and_justice.rs:70`, consumed by
  `Effect::AddCounter { target: EffectTarget::DeclaredTarget { index: 0 }, counter: CounterType::PlusOnePlusOne, count: 1 }`
  at `:62-66`. `TargetRequirement::TargetCreature` (`card_definition.rs:2946`) carries no
  controller restriction, so **any** creature on the battlefield is a legal recipient.
  **Effect on play**: the engine's auto-choice can grow an *opponent's* creature — in a 4-player
  game the counter can land on a creature the equipped creature's controller does not control,
  which the printed card forbids outright. This is independent of the auto-choice: even a human
  choosing correctly, the legal-target set is wrong (an opponent could be forced to be offered as
  a legal option, and any bot/heuristic picking from it produces an illegal board state).
  Authorable today — `TargetRequirement::TargetCreatureWithFilter(TargetFilter { controller: TargetController::You, .. })`
  (`card_definition.rs:2966`) is the standard idiom, used e.g. in `roalesk_apex_hybrid.rs:43-46`
  for the same "+1/+1 counter on a creature you control, then proliferate" shape.
- WATCH: `Equip {2}` cost unencoded (systemic, see header). Secondary and NOT counted as a defect:
  the printed clause is a *choice*, not a *target* ("put a +1/+1 counter on a creature you
  control" uses no "target"), so authoring it as `targets` also makes it subject to
  hexproof/shroud/protection and to fizzling, which the printed card is not. Recording as WATCH
  because that is a DSL-expressiveness matter (there is no non-targeted "choose a creature you
  control" requirement), whereas the missing controller filter above is a plain authoring error.

### Sylvan Messenger — **CLASS B**
- declared completeness: `Complete` (BY `#[default]` — no field declared)
- MCP printed oracle text: "Trample (This creature can deal excess combat damage to the player or planeswalker it's attacking.)\nWhen this creature enters, reveal the top four cards of your library. Put all Elf cards revealed this way into your hand and the rest on the bottom of your library in any order."
- stored `oracle_text` field: **matches** (verbatim, including the Trample reminder text)
- verdict rationale: `{3}{G}`, `Creature — Elf`, 2/2 all correct. `KeywordAbility::Trample`
  present. The ETB is a `RevealAndRoute` with `count: Fixed(4)`, `filter: has_subtype "Elf"`,
  `matched_dest: Hand`, `unmatched_dest: Library { position: Bottom }` — every printed clause
  encoded. The only deviation is that the non-Elf cards reach the bottom in a fixed order rather
  than a controller-chosen one ("in any order"), which is exactly a runtime auto-choice among
  legal options; the def carries an explicit in-file note (lines 7-9) saying so and citing
  `goblin_ringleader.rs` as the shipped precedent.

### Tainted Observer — **CLASS B**
- declared completeness: `Complete` (BY `#[default]` — no field declared)
- MCP printed oracle text: "Flying\nToxic 1 (Players dealt combat damage by this creature also get a poison counter.)\nWhenever another creature you control enters, you may pay {2}. If you do, proliferate. (Choose any number of permanents and/or players, then give each another counter of each kind already there.)"
- stored `oracle_text` field: **matches** (verbatim, both reminder texts included)
- verdict rationale: `{1}{G}{U}`, `Creature — Phyrexian Bird`, 2/3 all correct;
  `creature_types(&["Phyrexian", "Bird"])` reproduces the printed subtype line exactly (no
  `Horror` — correct, that is Thrummingbird's line, not this one). `Flying` and `Toxic(1)`
  present. The trigger correctly encodes **all three** restrictive qualifiers: `another`
  (`exclude_self: true`), `you control` (`controller: TargetController::You`), and the optional
  pay (`Effect::MayPayThenEffect { cost: Cost::Mana({2}), payer: PlayerTarget::Controller, then: Proliferate }`)
  — the "you may" is preserved, not dropped, and the payer is the controller.

### Tezzeret's Gambit — **CLASS B**
- declared completeness: `Complete` (BY `#[default]` — no field declared)
- MCP printed oracle text: "({U/P} can be paid with either {U} or 2 life.)\nDraw two cards, then proliferate. (Choose any number of permanents and/or players, then give each another counter of each kind already there.)"
- stored `oracle_text` field: **DIFFERS (cosmetic only)** — "({U/P} can be paid with either {U} or 2 life.)\nDraw two cards, then proliferate." (proliferate reminder text omitted; rules text matches)
- verdict rationale: Mana cost is right and non-trivially so — `generic: 3` plus
  `phyrexian: vec![PhyrexianMana::Single(ManaColor::Blue)]` is `{3}{U/P}`, not a hybrid
  approximation. `Sorcery`, no P/T. Effect is `Sequence([DrawCards { player: Controller, count: 2 }, Proliferate])`
  in the printed order. Nothing optional was dropped; nothing unprinted was added.
- WATCH: reminder-text omission in the stored `oracle_text` only. Cosmetic — three defs in this
  batch (this one, Sword of Truth and Justice, Yawgmoth) omit the proliferate parenthetical while
  Thrummingbird, Tainted Observer, Thirsting Roots and Unnatural Restoration include it. Worth a
  corpus-wide consistency sweep, not a per-card defect.

### Thirsting Roots — **CLASS B**
- declared completeness: `Complete` (BY `#[default]` — no field declared)
- MCP printed oracle text: "Choose one —\n• Search your library for a basic land card, reveal it, put it into your hand, then shuffle.\n• Proliferate. (Choose any number of permanents and/or players, then give each another counter of each kind already there.)"
- stored `oracle_text` field: **matches** (verbatim, including reminder text)
- verdict rationale: `{G}` Sorcery. `ModeSelection { min_modes: 1, max_modes: 1, allow_duplicate_modes: false }`
  is the correct encoding of "Choose one —". Mode 0 is
  `SearchLibrary(basic_land_filter(), dest: Hand)` followed by `Effect::Shuffle { player: Controller }`
  — the printed "then shuffle" is a separate, present step, not silently folded into
  `shuffle_before_placing` (which is correctly `false`). Mode 1 is `Proliferate`. Both the mode
  choice and the search choice are runtime auto-choices — the BASELINE premise.
- WATCH: `reveal: true` on `Effect::SearchLibrary` is **inert** — the engine destructures
  `reveal: _`. Pre-existing and already tracked as **OOS-DP9-9**; noted here only because the
  printed card says "reveal it" and a `Complete` marker silently covers that unimplemented
  clause. Same shape as the `inventors_fair` note added in PB-DX3.

### Thrasios, Triton Hero — **CLASS D**
- declared completeness: `Complete` (BY `#[default]` — no field declared)
- MCP printed oracle text: "{4}: Scry 1, then reveal the top card of your library. If it's a land card, put it onto the battlefield tapped. Otherwise, draw a card.\nPartner (You can have two commanders if both have partner.)"
- stored `oracle_text` field: **matches** (verbatim, including the Partner reminder text)
- verdict rationale: `{G}{U}`, `Legendary Creature — Merfolk Wizard` (supertype **present** via
  `full_types(&[SuperType::Legendary], ...)`), 1/3, `KeywordAbility::Partner` — all correct.
  `Cost::Mana({4})`, `Effect::Scry { count: 1 }`, and the land half
  (`matched_dest: ZoneTarget::Battlefield { tapped: true }`) are all faithful. The defect is the
  **"otherwise" half**: the printed card performs a *draw*, and the def performs a *zone move*.
- **DEFECT 1**: printed "Otherwise, **draw a card**." vs def
  `unmatched_dest: ZoneTarget::Hand { owner: PlayerTarget::Controller }` at
  `crates/card-defs/src/defs/thrasios_triton_hero.rs:48-50`, inside the
  `Effect::RevealAndRoute` at `:40-51`. `RevealAndRoute` routes the revealed object with a
  zone move; it emits no draw. **Effect on play**: the card lands in hand — the same *zone* —
  but the game never sees a draw event, so nothing that keys on drawing applies. Concretely:
  "whenever you draw a card" triggers never fire; draw *replacement* effects never apply, so an
  opponent's Notion Thief / Leovold does not steal or stop it, and a Narset/Hullbreacher-style
  draw restriction does not bite; PB-DP5's `WouldDraw` / dredge channel is bypassed entirely; and
  a controller under a "you can't draw cards" effect still gets the card. Thrasios is a
  first-rank Commander card whose activated ability is used many times per game, so the
  divergence compounds. This is orthogonal to any auto-choice — the auto-choice here is the
  Scry 1 keep/bottom decision, which is legitimately class B.
- WATCH: the def has **no in-file note** acknowledging the approximation, unlike Sylvan
  Messenger in this same batch (lines 7-9) which documents its own. An undocumented
  approximation under a `#[default]`-derived `Complete` marker is the harder case to find later.

### Thrummingbird — **CLASS B**
- declared completeness: `Complete` (BY `#[default]` — no field declared)
- MCP printed oracle text: "Flying\nWhenever this creature deals combat damage to a player, proliferate. (Choose any number of permanents and/or players, then give each another counter of each kind already there.)"
- stored `oracle_text` field: **matches** (verbatim, including reminder text)
- verdict rationale: `{1}{U}`, `Creature — Phyrexian Bird Horror` — `creature_types(&["Phyrexian", "Bird", "Horror"])`
  reproduces all three printed subtypes — 1/1. `Flying` present.
  `TriggerCondition::WhenDealsCombatDamageToPlayer` with `effect: Effect::Proliferate` is a
  one-to-one encoding. Nothing printed is missing; nothing unprinted is added. The only runtime
  auto-choice is which permanents/players proliferate picks, which is the BASELINE premise.

### Unnatural Restoration — **CLASS B**
- declared completeness: `Complete` (BY `#[default]` — no field declared)
- MCP printed oracle text: "Return target permanent card from your graveyard to your hand. Proliferate. (Choose any number of permanents and/or players, then give each another counter of each kind already there.)"
- stored `oracle_text` field: **matches** (verbatim, including reminder text)
- verdict rationale: `{1}{G}` Sorcery. `TargetRequirement::TargetCardInYourGraveyard` correctly
  scopes to **your** graveyard (not any graveyard), and the `has_card_types` OR-list
  `[Creature, Artifact, Enchantment, Land, Planeswalker]` is the right expansion of "permanent
  card". `MoveZone` to `Hand { owner: Controller }` then `Proliferate`, in the printed order.
- WATCH: the permanent-type list omits `Battle`. The Battle subsystem is a declared project
  deferral (CLAUDE.md open-seed list), and no `Battle` cards exist in the corpus, so this is
  presently unobservable. Recording it so the list is revisited if Battles are ever added — this
  is a filter that will silently go stale, not one that is wrong today.

### Urza's Incubator — **CLASS B**
- declared completeness: `Complete` (BY `#[default]` — no field declared)
- MCP printed oracle text: "As this artifact enters, choose a creature type.\nCreature spells of the chosen type cost {2} less to cast."
- stored `oracle_text` field: **matches** (verbatim)
- verdict rationale: `{3}` Artifact, no P/T. The "as this enters, choose a creature type" clause
  is correctly a **self-replacement** (`AbilityDefinition::Replacement` with
  `ReplacementTrigger::WouldEnterBattlefield`, `is_self: true`,
  `ReplacementModification::ChooseCreatureType`) per CR 614.1c, not a triggered ability — that
  distinction matters and the def gets it right. The cost reduction is
  `SpellCostModifier { change: -2, filter: SpellCostFilter::HasChosenCreatureSubtype, scope: CostModifierScope::AllPlayers }`.
  **`AllPlayers` is correct and deliberate**: the printed clause has no "you cast" qualifier, so
  it reduces every player's matching creature spells; the def carries an explicit comment
  (lines 18-21) arguing exactly that. The hardcoded `SubType("Human")` argument is the engine's
  auto-choice standing in for the player's type choice — the BASELINE premise, class B.
- WATCH: the auto-chosen type is the literal `"Human"` rather than anything derived, so in play
  this card only ever discounts Humans. That is the recorded auto-choice, not a def defect, but
  it is a maximally-arbitrary default (contrast a "choose the most common type in your deck"
  heuristic), and it is shared verbatim with Vanquisher's Banner below.

### Vanquisher's Banner — **CLASS B**
- declared completeness: `Complete` (BY `#[default]` — no field declared)
- MCP printed oracle text: "As this artifact enters, choose a creature type.\nCreatures you control of the chosen type get +1/+1.\nWhenever you cast a creature spell of the chosen type, draw a card."
- stored `oracle_text` field: **matches** (verbatim)
- verdict rationale: `{5}` Artifact — mana cost correct (this card is frequently misremembered as
  `{4}`). All three printed clauses present: the CR 614.1c self-replacement type choice; a
  `PtModify` / `ModifyBoth(1)` static filtered by `EffectFilter::CreaturesYouControlOfChosenType`
  (the "you control" restriction **is** carried, unlike Sword of Truth and Justice above); and a
  `WheneverYouCastSpell` trigger with `spell_type_filter: Some(vec![CardType::Creature])` and
  `chosen_subtype_filter: true` drawing 1 for the controller. `noncreature_only: false` and
  `during_opponent_turn: false` are the correct settings for an unqualified "whenever you cast".
  Only the type choice is auto-made.

### Victimize — **CLASS B**
- declared completeness: `Complete` (BY `#[default]` — no field declared)
- MCP printed oracle text: "Choose two target creature cards in your graveyard. Sacrifice a creature. If you do, return the chosen cards to the battlefield tapped."
- stored `oracle_text` field: **matches** (verbatim)
- verdict rationale: `{2}{B}` Sorcery. Two `TargetCardInYourGraveyard(has_card_type: Creature)`
  requirements = "two target creature cards in your graveyard". The sacrifice is authored
  **mandatory** (`Effect::SacrificePermanents`, no `may` wrapper), which is correct — the
  2020-11-10 ruling makes it mandatory-if-able — and the "If you do" is properly gated by
  `Condition::SacrificeFired` rather than assumed, so a controller with no other creature does
  not get the return. Both returns are `ZoneTarget::Battlefield { tapped: true }` — the printed
  "tapped" is carried. `controller_override: Some(PlayerTarget::Controller)` is right (they are
  your own graveyard cards, so owner and controller coincide). Which creature is sacrificed is
  the runtime auto-choice — the BASELINE premise. The def carries a thorough CR 608.2c/608.2h
  rationale block (lines 5-14) that is accurate against the ruling.
- WATCH: the def does not encode target **distinctness**. CR 601.2c forbids choosing the same
  object for two instances of "target" in one spell; the DSL has
  `TargetRequirement::TargetPermanentDistinctFrom(usize)` (`card_definition.rs:2997`) but there
  appears to be no graveyard-card counterpart, and I did not establish whether
  `validate_targets_inner` enforces distinctness globally. If it does not, a controller with a
  single creature card in the graveyard could name it twice. Flagged as a question for the
  engine, not asserted as a def defect — this is a DSL-expressiveness / engine-validation matter
  and would apply to every multi-target-in-one-zone def, not just this one.

### Yawgmoth, Thran Physician — **CLASS B**
- declared completeness: `Complete` (BY `#[default]` — no field declared)
- MCP printed oracle text: "Protection from Humans\nPay 1 life, Sacrifice another creature: Put a -1/-1 counter on up to one target creature and draw a card.\n{B}{B}, Discard a card: Proliferate. (Choose any number of permanents and/or players, then give each another counter of each kind already there.)"
- stored `oracle_text` field: **DIFFERS (cosmetic only)** — "Protection from Humans\nPay 1 life, Sacrifice another creature: Put a -1/-1 counter on up to one target creature and draw a card.\n{B}{B}, Discard a card: Proliferate." (proliferate reminder text omitted; rules text matches)
- verdict rationale: `{2}{B}{B}`, `Legendary Creature — Human Cleric` (supertype **present**),
  2/4 — all correct. `ProtectionFrom(FromSubType("Human"))` encodes "Protection from Humans".
  First ability: `Cost::Sequence([PayLife(1), Sacrifice(TargetFilter { has_card_type: Creature, controller: You, exclude_self: true })])`
  — the printed "**another**" is carried by `exclude_self: true` (so Yawgmoth cannot eat himself,
  CR 109.1), and **"up to one target creature"** is correctly
  `TargetRequirement::UpToN { count: 1, inner: TargetCreature }`, so the ability is activatable
  with zero targets exactly as printed. Note the `-1/-1` counter here is genuinely permanent —
  the printed text says "Put a -1/-1 counter", with no duration — so this is **not** the
  Shambling Ghast shape despite the same counter type. Second ability:
  `Cost::Sequence([Mana({B}{B}), DiscardCard])` → `Effect::Proliferate`. Both abilities present,
  both costs complete; the runtime auto-choices are which creature is sacrificed, which card is
  discarded, and the proliferate selection — all BASELINE premise.
- WATCH: reminder-text omission in stored `oracle_text` only (cosmetic; see the Tezzeret's
  Gambit note). Also: the def's trailing comment block (lines 90-94) asserts "Complete", but the
  def declares no `completeness` field — the claim is true only via the `#[default]` derive. Same
  shape as the two defs that have bitten this project; recording it because a prose claim of
  completeness next to an undeclared marker reads as an explicit declaration and is not one.

---

## SUMMARY batch7: 11 class-B, 2 class-D (Sword of Truth and Justice, Thrasios, Triton Hero), 11 watch

**Class D detail**

1. **Sword of Truth and Justice** — printed "put a +1/+1 counter on **a creature you control**"
   is authored as an unrestricted `TargetRequirement::TargetCreature`
   (`sword_of_truth_and_justice.rs:70`), so the +1/+1 counter can legally land on an opponent's
   creature. Fixable today with `TargetCreatureWithFilter(TargetFilter { controller: TargetController::You, .. })`,
   the idiom already used by `roalesk_apex_hybrid.rs:43-46` for the identical printed shape.

2. **Thrasios, Triton Hero** — printed "Otherwise, **draw a card**" is authored as
   `unmatched_dest: ZoneTarget::Hand` (`thrasios_triton_hero.rs:48-50`), a zone move rather than
   a draw, so no draw event occurs: draw triggers do not fire, draw replacements (Notion Thief,
   Leovold, Hullbreacher) do not apply, the dredge/`WouldDraw` channel is bypassed, and
   "can't draw" restrictions do not bite.

**Watch items by card** (none asserted as defects): Sword of Feast and Famine (equip cost),
Sword of Truth and Justice (equip cost; choice-authored-as-target), Tezzeret's Gambit (reminder
text), Thirsting Roots (inert `reveal: true`, OOS-DP9-9), Thrasios (undocumented approximation),
Unnatural Restoration (`Battle` absent from permanent-type list), Urza's Incubator (hardcoded
`"Human"` auto-choice), Victimize (target distinctness unverified), Yawgmoth (reminder text;
prose "Complete" with no declared marker), plus the two batch-level items in the header
(13/13 undeclared `completeness`; corpus-wide unencoded Equip cost).
