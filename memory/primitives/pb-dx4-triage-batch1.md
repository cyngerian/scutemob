# PB-DX4 — class-B / class-D triage of the PB-DP10 `BASELINE`, batch 1 of 7

**Date**: 2026-08-01
**Scope**: 14 defs. Read-only triage. MCP (`mcp__mtg-rules__lookup_card`) is the authority for
printed oracle text, type line, mana cost, P/T and keywords; the def's own `oracle_text` field
is *evidence under test*, not a source.

**Reminder of the classification** (from the dispatch brief):
- **class B** — def faithfully encodes the printed card; the only deviation is that the engine
  auto-picks among legal options at runtime. Expected; this is what `BASELINE` exists to record.
- **class D** — the def is wrong against oracle text *independently* of the auto-choice.

A note that applies to **12 of these 14 defs**: they declare **no `completeness` field at all** and
are therefore `Complete` via the `#[default]` derive on `Completeness`. Per the `aurelia_the_warleader`
(PB-DX1) and `emeria_the_sky_ruin` (PB-DX3b) precedents this is a known silent-defect generator, so
each entry below records it explicitly. In this batch it did **not** conceal a defect — every one of
the 12 is oracle-correct — but the count is worth carrying to the remaining six batches.

---

### Accursed Marauder — **CLASS B**
- declared completeness: `Complete` (BY `#[default]` — no field declared; `..Default::default()` at
  `accursed_marauder.rs:41`)
- MCP printed oracle text: "When this creature enters, each player sacrifices a nontoken creature of their choice."
- MCP type line / cost / P/T: `Creature — Zombie Warrior`, `{1}{B}`, 2/1 — def matches
  (`creature_types(&["Zombie", "Warrior"])`, `generic: 1, black: 1`, `power: Some(2), toughness: Some(1)`).
- stored `oracle_text` field: DIFFERS (cosmetic) — "When this enters, each player sacrifices a nontoken creature."
- verdict rationale: every printed clause is encoded — `TriggerCondition::WhenEntersBattlefield`,
  `Effect::SacrificePermanents { player: EachPlayer, count: Fixed(1), filter: Creature + is_nontoken }`.
  "each player" (not "each opponent") is correct, the nontoken restriction is present, and the only
  runtime deviation is that the engine picks *which* creature each player sacrifices — precisely the
  "of their choice" clause `BASELINE` is meant to record.
- WATCH: the stored `oracle_text` at `accursed_marauder.rs:15` drops both the modern "this creature"
  wording and the trailing "of their choice". Cosmetic — the *behaviour* is faithful and the dropped
  clause is exactly the auto-choice `BASELINE` documents — but it is a string mismatch and a
  text-field sweep should normalize it.

### Anowon, the Ruin Sage — **CLASS B**
- declared completeness: `Complete` (BY `#[default]` — no field declared; `anowon_the_ruin_sage.rs:47`)
- MCP printed oracle text: "At the beginning of your upkeep, each player sacrifices a non-Vampire creature of their choice."
- MCP type line / cost / P/T: `Legendary Creature — Vampire Shaman`, `{3}{B}{B}`, 4/3 — def matches,
  and the `Legendary` supertype **is** present (`full_types(&[SuperType::Legendary], ...)`).
- stored `oracle_text` field: matches (verbatim, including "of their choice")
- verdict rationale: `AtBeginningOfYourUpkeep` + `SacrificePermanents { player: EachPlayer, count: 1,
  filter: Creature with exclude_subtypes: [Vampire] }` is a clause-for-clause encoding. "each player"
  correctly includes Anowon's own controller. Only the per-player pick is engine-made.

### Atomize — **CLASS B**
- declared completeness: `Complete` (BY `#[default]` — no field declared; `atomize.rs:37`)
- MCP printed oracle text: "Destroy target nonland permanent. Proliferate. (Choose any number of permanents and/or players, then give each another counter of each kind already there.)"
- MCP type line / cost: `Instant`, `{2}{B}{G}` — def matches.
- stored `oracle_text` field: matches (verbatim, reminder text included)
- verdict rationale: `Effect::Sequence([DestroyPermanent{DeclaredTarget 0}, Proliferate])` in printed
  order, with `TargetPermanentWithFilter(TargetFilter { non_land: true })` — the correct nonland
  filter (KI-1 clean, not a bare `TargetPermanent`). `Proliferate`'s "any number" choice is the
  engine-made half.

### Atraxa, Praetors' Voice — **CLASS B**
- declared completeness: `Complete` (BY `#[default]` — no field declared; `atraxa_praetors_voice.rs:44`)
- MCP printed oracle text: "Flying, vigilance, deathtouch, lifelink\nAt the beginning of your end step, proliferate. (Choose any number of permanents and/or players, then give each another counter of each kind already there.)"
- MCP type line / cost / P/T: `Legendary Creature — Phyrexian Angel Horror`, `{G}{W}{U}{B}`, 4/4 — def
  matches; `Legendary` present; all three subtypes present in printed order.
- stored `oracle_text` field: matches (verbatim)
- verdict rationale: all four printed keywords are present as separate `AbilityDefinition::Keyword`
  entries, none extra; `AtBeginningOfYourEndStep` + `Effect::Proliferate`. Only the proliferate
  selection is engine-made.

### Birthing Ritual — **CLASS B** (with the batch's strongest WATCH)
- declared completeness: `Complete` (**explicit** — `completeness: Completeness::Complete` at `birthing_ritual.rs:79`)
- MCP printed oracle text: "At the beginning of your end step, if you control a creature, look at the top seven cards of your library. Then you may sacrifice a creature. If you do, you may put a creature card with mana value X or less from among those cards onto the battlefield, where X is 1 plus the sacrificed creature's mana value. Put the rest on the bottom of your library in a random order."
- MCP type line / cost: `Enchantment`, `{1}{G}` — def matches.
- stored `oracle_text` field: matches (verbatim)
- verdict rationale: every printed clause has a structural counterpart — the CR 603.4 intervening-if
  is real (`Condition::YouControlNOrMoreWithFilter { count: 1, filter: Creature }`, lines 45-51, i.e.
  this is *not* another stale-`intervening_if: None` case); `count: Fixed(7)`; the "1 plus the
  sacrificed creature's mana value" arithmetic is encoded exactly as
  `Sum(Fixed(1), ManaValueOfSacrificedCreature)`; `rest_to` is library-**bottom**; the second "may"
  is `optional: true`. The first "may" — "Then you may sacrifice a creature" — is encoded as
  `place_cost: Some(Cost::Sacrifice(Creature))`, i.e. it *is* represented in the DSL and the engine
  auto-decides to pay it. That is an auto-choice, not a dropped clause, which is the line between
  this and Smuggler's Copter (whose optionality has no representation whatsoever). Hence B.
- WATCH 1 (the reason this is borderline): the def's own comment at `birthing_ritual.rs:14-16` states
  the policy plainly — *"the sacrifice fires whenever a creature is available, even into a whiff,
  same as every other MayPayThenEffect-shaped Complete card."* Combined with `optional: true` on the
  placement, the engine can pay the sacrifice and then decline to place, i.e. lose a creature for
  nothing, every end step. If PB-DX4's owner rules that "auto-paying a printed *may*-cost" counts as
  class D, this def flips — but so does a whole documented class of `Complete` cards, so it is a
  **policy** call, not a defect in this file, and I have not reported it as D.
- WATCH 2: "Put the rest on the bottom of your library **in a random order**" is realized as
  deterministic `ObjectId`-ascending placement (comment at `birthing_ritual.rs:18-20`). This is the
  standing M7 no-`rand` precedent shared with `RevealAndRoute`/`Scry`/`PutOnLibrary`, not a
  Birthing-Ritual-specific deviation.

### Blightbelly Rat — **CLASS B**
- declared completeness: `Complete` (BY `#[default]` — no field declared; `blightbelly_rat.rs:32`)
- MCP printed oracle text: "Toxic 1 (Players dealt combat damage by this creature also get a poison counter.)\nWhen this creature dies, proliferate. (Choose any number of permanents and/or players, then give each another counter of each kind already there.)"
- MCP type line / cost / P/T: `Creature — Phyrexian Rat`, `{1}{B}`, 2/2 — def matches.
- stored `oracle_text` field: DIFFERS (cosmetic) — "Toxic 1\nWhen Blightbelly Rat dies, proliferate."
  (pre-2024 self-reference wording; both reminder-text parentheticals omitted).
- verdict rationale: `KeywordAbility::Toxic(1)` with the printed value 1, and
  `TriggerCondition::WhenDies` + `Effect::Proliferate`. Trigger condition matches the printed
  "dies" (contrast Shambling Ghast, whose stored text said "enters" against a `WhenDies` trigger —
  here both text and trigger agree on death). No phantom keywords.
- WATCH: cosmetic `oracle_text` drift only, `blightbelly_rat.rs:16`.

### Bloated Contaminator — **CLASS B**
- declared completeness: `Complete` (BY `#[default]` — no field declared; `bloated_contaminator.rs:38`)
- MCP printed oracle text: "Trample\nToxic 1 (Players dealt combat damage by this creature also get a poison counter.)\nWhenever this creature deals combat damage to a player, proliferate. (Choose any number of permanents and/or players, then give each another counter of each kind already there.)"
- MCP type line / cost / P/T: `Creature — Phyrexian Beast`, `{2}{G}`, 4/4 — def matches.
- stored `oracle_text` field: matches (verbatim, both reminder texts included)
- verdict rationale: `Trample` + `Toxic(1)` keywords, then
  `WhenDealsCombatDamageToPlayer` + `Effect::Proliferate`. "to a player" — not "to an opponent" —
  is faithfully unrestricted, matching print. Only the proliferate selection is engine-made.

### Bolt Bend — **CLASS B**
- declared completeness: `Complete` (BY `#[default]` — no field declared; `bolt_bend.rs:37`)
- MCP printed oracle text: "This spell costs {3} less to cast if you control a creature with power 4 or greater.\nChange the target of target spell or ability with a single target."
- MCP type line / cost: `Instant`, `{3}{R}` — def matches (`generic: 3, red: 1`; the printed cost is
  the *full* cost, correctly stored, with the reduction as a separate field rather than baked in).
- stored `oracle_text` field: matches (verbatim)
- verdict rationale: `self_cost_reduction: SelfCostReduction::ConditionalPowerThreshold { threshold: 4,
  reduction: 3 }` encodes the printed threshold and amount exactly ("power 4 or greater" → 4, "{3}
  less" → 3). `Effect::ChangeTargets { must_change: true }` against
  `TargetRequirement::TargetSpellOrAbilityWithSingleTarget` matches CR 115.7a/b, and the def comment
  correctly records that the original target stands when no other legal target exists. The engine
  picking *which* new target it becomes is the auto-choice.

### Brainstorm — **CLASS B**
- declared completeness: `Complete` (BY `#[default]` — no field declared; `brainstorm.rs:35`)
- MCP printed oracle text: "Draw three cards, then put two cards from your hand on top of your library in any order."
- MCP type line / cost: `Instant`, `{U}` — def matches.
- stored `oracle_text` field: matches (verbatim)
- verdict rationale: `Sequence([DrawCards{Controller, 3}, PutOnLibrary{Controller, 2, from: Hand}])` —
  correct counts, correct order, correct player. The whole point of the card ("in any order", and
  *which* two) is the engine-made choice: the def comment at `brainstorm.rs:2` records the
  deterministic first-2-by-`ObjectId` policy. That is exactly the class-B shape.

### Burglar Rat — **CLASS B**
- declared completeness: `Complete` (BY `#[default]` — no field declared; `burglar_rat.rs:34`)
- MCP printed oracle text: "When this creature enters, each opponent discards a card."
- MCP type line / cost / P/T: `Creature — Rat`, `{1}{B}`, 1/1 — def matches.
- stored `oracle_text` field: matches (verbatim)
- verdict rationale: `WhenEntersBattlefield` + `ForEach { over: EachOpponent, ... DiscardCards { count: 1 } }`.
  **`EachOpponent`, not `EachPlayer`** — the controller is correctly excluded. `PlayerTarget::DeclaredTarget
  { index: 0 }` inside a `ForEach` is the established correct idiom for "the player this iteration is
  over" and is not a defect. Only *which* card each opponent discards is engine-made.

### Butcher of Malakir — **CLASS B**
- declared completeness: `Complete` (BY `#[default]` — no field declared; `butcher_of_malakir.rs:52`)
- MCP printed oracle text: "Flying\nWhenever this creature or another creature you control dies, each opponent sacrifices a creature of their choice."
- MCP type line / cost / P/T: `Creature — Vampire Warrior`, `{5}{B}{B}`, 5/4 — def matches. Correctly
  **not** Legendary.
- stored `oracle_text` field: DIFFERS (cosmetic) — self-reference by name ("Whenever Butcher of
  Malakir or another creature you control dies…") rather than the modern "this creature"; otherwise
  clause-identical, including "of their choice".
- verdict rationale: this is the KI-9 shape and it passes. The printed trigger is *"this creature or
  another creature you control"* — i.e. self **included**, restricted to your creatures — and the def
  says exactly that: `WheneverCreatureDies { controller: Some(TargetController::You),
  exclude_self: false, nontoken_only: false, filter: None }`. `exclude_self: false` is right *because*
  print says "this creature or another"; an `exclude_self: true` here would have been the defect.
  Effect is `ForEach { EachOpponent, SacrificePermanents { creature filter, count 1 } }` —
  "each opponent", not "each player", matching print. Only the per-opponent pick is engine-made.
- WATCH: cosmetic `oracle_text` self-reference drift only, `butcher_of_malakir.rs:17-19`.

### Cached Defenses — **CLASS B**
- declared completeness: `Complete` (BY `#[default]` — no field declared; `cached_defenses.rs:26`)
- MCP printed oracle text: "Bolster 3. (Choose a creature with the least toughness among creatures you control and put three +1/+1 counters on it.)"
- MCP type line / cost: `Sorcery`, `{2}{G}` — def matches.
- stored `oracle_text` field: matches (verbatim, reminder text included)
- verdict rationale: `Effect::Bolster { player: Controller, count: Fixed(3) }` — correct keyword,
  correct amount, correct controller. Bolster's own tie-break ("if two or more creatures are tied for
  least toughness, you choose one") is the engine-made half and is the sole reason this def is in
  `BASELINE`.

### Caged Sun — **CLASS B**
- declared completeness: `Complete` (BY `#[default]` — no field declared; `caged_sun.rs:64`)
- MCP printed oracle text: "As this artifact enters, choose a color.\nCreatures you control of the chosen color get +1/+1.\nWhenever a land's ability causes you to add one or more mana of the chosen color, add an additional one mana of that color."
- MCP type line / cost: `Artifact`, `{6}` — def matches (`types(&[CardType::Artifact])`, `generic: 6`;
  correctly no supertype and no P/T).
- stored `oracle_text` field: DIFFERS (cosmetic) — "As Caged Sun enters" for the modern "As this
  artifact enters"; clauses 2 and 3 verbatim.
- verdict rationale: all three printed clauses present and none extra. Clause 1 is a CR 614.12
  *replacement*, not a trigger — correctly modelled as `ReplacementModification::ChooseColor`, and the
  `Color::White` argument is the engine's auto-choice placeholder (the class-B deviation). Clause 2 is
  `EffectLayer::PtModify` / `ModifyBoth(1)` over `EffectFilter::CreaturesYouControlOfChosenColor`
  with `WhileSourceOnBattlefield` — correct layer, correct +1/+1, correct "you control" restriction,
  and it reads the chosen colour dynamically rather than freezing it. Clause 3's restriction to
  `ReplacementManaSourceFilter::AnyLand` matches the printed "a **land's** ability" (a common
  mis-authoring would have been any mana source), and `AddOneManaOfChosenColor` matches "an
  additional **one** mana" against the printed "one **or more**" trigger condition.
- WATCH: clause 3 is printed as a *triggered mana ability* and is implemented as an additive
  **replacement** on `ManaWouldBeProduced` (`caged_sun.rs:50-62`). The def comment argues this from
  CR 605.3 (mana abilities are stackless) and calls it the engine-wide PB-E pattern. Mana output is
  identical; the observable difference would only be a "whenever a triggered ability triggers"-style
  interaction. Engine-architecture deviation, not an oracle-text defect, and not specific to this def.

### Cankerbloom — **CLASS B**
- declared completeness: `Complete` (**explicit** — `completeness: Completeness::Complete` at `cankerbloom.rs:82`)
- MCP printed oracle text: "{1}, Sacrifice this creature: Choose one —\n• Destroy target artifact.\n• Destroy target enchantment.\n• Proliferate. (Choose any number of permanents and/or players, then give each another counter of each kind already there.)"
- MCP type line / cost / P/T: `Creature — Phyrexian Fungus`, `{1}{G}`, 3/2 — def matches.
- stored `oracle_text` field: matches (verbatim, including the bullet structure and reminder text)
- verdict rationale: cost is `Cost::Sequence([Mana{generic:1}, SacrificeSelf])` — both printed cost
  components, in order. `ModeSelection { min_modes: 1, max_modes: 1, allow_duplicate_modes: false }`
  is the correct encoding of "Choose one —". All three modes present in printed order with the right
  effects, and `mode_targets` gives mode 0 `TargetArtifact`, mode 1 `TargetEnchantment`, mode 2 an
  empty slice — the CR 700.2c handling, which is the *correct* reading (activating the proliferate
  mode must not require an artifact or enchantment on the battlefield). The `effect: Sequence(vec![])`
  at `cankerbloom.rs:46` is a documented placeholder superseded by `modes`, not a no-op stub of the
  KI-10 kind. Engine-made halves: which mode, and the proliferate selection.

---

SUMMARY batch1: 14 class-B, 0 class-D (none), 8 watch
