# PB-DX4 class-B/class-D triage — batch 4 of 7

**Date**: 2026-08-01
**Cards**: 14
**Method**: MCP `lookup_card` (authoritative printed oracle text / type line / mana cost / P-T /
keywords) read against the def file in full, clause by clause. Read-only; no file was edited.

Two supporting engine facts were verified by reading source rather than assumed, because both
class-D calls turn on them:

- `Effect::RevealAndRoute` **routes ALL matches**, not one. Stated verbatim in the DSL's own
  doc comment on the sibling variant: `crates/card-types/src/cards/card_definition.rs:2040-2041`
  — *"The put-≤1 sibling of `Effect::RevealAndRoute` (which routes ALL matches and has no
  cost/gate)"*. The put-≤1 primitive (`Effect::LookAtTopThenPlace`, with an `optional: bool`)
  exists and is used elsewhere in this very batch (Growing Rites of Itlimoc).
- `TriggerCondition::WhenDealsCombatDamageToPlayer` lowers to
  `TriggerEvent::SelfDealsCombatDamageToPlayer`, and the only dispatch site is gated by
  `if matches!(assignment.target, CombatDamageTarget::Player(_))` —
  `crates/engine/src/rules/abilities.rs:5099-5104`. Combat damage to a **planeswalker** does not
  fire it.
- `Completeness` derives `#[default] Complete`
  (`crates/card-types/src/cards/card_definition.rs:197-200`), so a def with no `completeness`
  field is deck-legal. Recorded per card below — this has bitten the project twice
  (`aurelia_the_warleader`, `emeria_the_sky_ruin`).

---

### Grateful Apparition — **CLASS D**
- declared completeness: `Complete` (BY `#[default]` — no field declared; `..Default::default()` at grateful_apparition.rs:35)
- MCP printed oracle text: "Flying\nWhenever this creature deals combat damage to a player or planeswalker, proliferate. (Choose any number of permanents and/or players, then give each another counter of each kind already there.)"
- stored `oracle_text` field: **matches** MCP verbatim (including "or planeswalker")
- mana cost `{1}{W}` ✓, type `Creature — Spirit` ✓, P/T 1/1 ✓, Flying ✓
- verdict rationale: The stored `oracle_text` correctly says "a player **or planeswalker**", but
  the authored trigger covers only the player half. The def therefore contradicts its own recorded
  oracle text, and the omission is behaviourally live: proliferate is a core reason this card is
  played, and attacking a planeswalker is the other half of its printed design.
- **DEFECT 1**: printed **"Whenever this creature deals combat damage to a player or planeswalker, proliferate."** vs def `trigger_condition: TriggerCondition::WhenDealsCombatDamageToPlayer` at `crates/card-defs/src/defs/grateful_apparition.rs:26`. That variant is dispatched only under `matches!(assignment.target, CombatDamageTarget::Player(_))` (`crates/engine/src/rules/abilities.rs:5099-5104`). **Effect on play**: a Grateful Apparition that attacks a planeswalker and connects does **not** proliferate. Strictly narrower than printed — the controller silently loses a trigger they are entitled to.
- **WATCH**: I found no `TriggerCondition` variant for combat damage to a planeswalker
  (`WhenDealsCombatDamageToPlayer`, `WheneverCreatureYouControlDealsCombatDamageToPlayer`,
  `WhenEquippedCreatureDealsCombatDamageToPlayer`, `WhenEquippedCreatureDealsCombatDamage`,
  `WhenAnyCreatureDealsCombatDamageToOpponent` are the whole family). If that holds after a proper
  search, the correct disposition is a `Completeness::partial(...)` marker (removing the def from
  deck-legality), **not** an in-def repair. Classifying D is still right: the def is wrong against
  oracle text and is currently `Complete`.

### Grave Pact — **CLASS B**
- declared completeness: `Complete` (BY `#[default]` — no field declared)
- MCP printed oracle text: "Whenever a creature you control dies, each other player sacrifices a creature of their choice."
- stored `oracle_text` field: matches
- mana cost `{1}{B}{B}{B}` ✓, type `Enchantment` ✓, no P/T ✓
- verdict rationale: `WheneverCreatureDies { controller: Some(TargetController::You), filter: None }`
  is the printed trigger; `ForEach { over: EachOpponent }` is the correct reading of "each other
  player" in multiplayer (every player who is not the controller); the sacrifice filter is
  creature-only. The only engine-side choice is *which* creature each opponent sacrifices — the
  printed card says "of their choice", so this is exactly the auto-choice the BASELINE exists to
  record. `PlayerTarget::DeclaredTarget { index: 0 }` inside `ForEach` is the established correct
  idiom, not a bug.

### Greater Good — **CLASS B**
- declared completeness: `Complete` (BY `#[default]` — no field declared)
- MCP printed oracle text: "Sacrifice a creature: Draw cards equal to the sacrificed creature's power, then discard three cards."
- stored `oracle_text` field: matches
- mana cost `{2}{G}{G}` ✓, type `Enchantment` ✓
- verdict rationale: `Cost::Sacrifice(creature)` as an activation cost; draw count is
  `EffectAmount::PowerOfSacrificedCreature` (LKI power at sacrifice time, CR 608.2b); discard is a
  mandatory `Fixed(3)`, matching the printed mandatory clause; the `Sequence` orders draw before
  discard so freshly drawn cards are discardable, which is correct. Both auto-choices (which
  creature to sacrifice, which three cards to discard) are the BASELINE's premise.

### Grisly Salvage — **CLASS D**
- declared completeness: `Complete` (BY `#[default]` — no field declared; `..Default::default()` at grisly_salvage.rs:38)
- MCP printed oracle text: "Reveal the top five cards of your library. You may put a creature or land card from among them into your hand. Put the rest into your graveyard."
- stored `oracle_text` field: **matches** MCP verbatim
- mana cost `{B}{G}` ✓, type `Instant` ✓
- verdict rationale: the def uses the wrong primitive. `RevealAndRoute` sends **every** card
  matching the filter to `matched_dest`; the printed card puts **at most one** there, and only if
  the controller chooses to. Both the count and the optionality are lost, and the loss is in the
  player's favour, which makes it a real power-level bug rather than a harmless narrowing.
- **DEFECT 1**: printed **"You may put a creature or land card from among them into your hand."** vs def `Effect::RevealAndRoute { ..., filter: creature|land, matched_dest: ZoneTarget::Hand { owner: Controller }, unmatched_dest: Graveyard }` at `crates/card-defs/src/defs/grisly_salvage.rs:20-33` (the `matched_dest: Hand` binding is line 27). `RevealAndRoute` is documented as routing **ALL** matches (`crates/card-types/src/cards/card_definition.rs:2040-2041`). **Effect on play**: in a typical creature/land-dense deck a two-mana instant puts **three to five** creature and land cards into hand instead of one, and dumps only the non-creature-non-land remainder into the graveyard — inverting the card's entire design (it is printed as graveyard fuel, not card advantage).
- **DEFECT 2**: printed **"You may put"** vs the def, which has no optionality field at all — `Effect::RevealAndRoute` has no `optional` member (`crates/card-types/src/cards/card_definition.rs:2030-2036`). **Effect on play**: the controller cannot decline; with a graveyard-matters or delirium plan, being forced to take the creature out of the graveyard is a real cost the printed card lets you refuse.
- **NOTE (not a defect, disposition aid)**: the correct primitive already exists and is used in this
  same batch — `Effect::LookAtTopThenPlace` places **at most one** matching card and carries
  `optional: bool` (see `growing_rites_of_itlimoc.rs:63-79`). Its only mismatch is that Grisly
  Salvage *reveals* rather than merely looks; the placement semantics are exactly right.

### Growing Rites of Itlimoc — **CLASS B**
- declared completeness: `Complete` (**explicit**, growing_rites_of_itlimoc.rs:141)
- MCP printed type line: "Legendary Enchantment // Legendary Land", keywords ["Transform"].
  (MCP returns no per-face oracle text for this DFC; the front-face text — "When Growing Rites of
  Itlimoc enters, look at the top four cards of your library. You may reveal a creature card from
  among them and put it into your hand. Put the rest on the bottom of your library in any order. /
  At the beginning of your end step, if you control four or more creatures, transform Growing Rites
  of Itlimoc." — and the back-face text "{T}: Add {G}. / {T}: Add {G} for each creature you
  control." were checked against the type line, the Transform keyword and the def's own clauses.)
- stored `oracle_text` field: matches the front face; back face `CardFace.oracle_text` matches
- mana cost `{2}{G}` ✓; **both** faces carry `SuperType::Legendary` ✓ (front `Legendary Enchantment`
  at :29, back `Legendary Land` at :90); `power: None, toughness: None` ✓
- verdict rationale: front ETB is `LookAtTopThenPlace { count: 4, filter: creature,
  destination: Hand, rest_to: Library{Bottom}, optional: true }` — count, filter, destination,
  remainder zone and the "you may" are all present and correct. The end-step transform carries
  `intervening_if: Some(Condition::YouControlNOrMoreWithFilter { count: 4, filter: creature })`,
  the CR 603.4 gate the printed "if you control four or more creatures" requires. Back face has
  both mana abilities, the second scaling on creature count. The engine choices (which creature to
  reveal; the order the rest go to the bottom) are auto-chosen and are the BASELINE's premise —
  "in any order" is a genuine player choice the printed card grants, but it is choice-only.

### Hazoret's Monument — **CLASS B**
- declared completeness: `Complete` (BY `#[default]` — no field declared)
- MCP printed oracle text: "Red creature spells you cast cost {1} less to cast.\nWhenever you cast a creature spell, you may discard a card. If you do, draw a card."
- stored `oracle_text` field: matches
- mana cost `{3}` ✓, type `Legendary Artifact` ✓ — `SuperType::Legendary` present (:14)
- verdict rationale: the cost reduction is `SpellCostModifier { change: -1,
  filter: SpellCostFilter::ColorAndCreature(Red), scope: Controller }` — compound (red **and**
  creature) and controller-scoped, both correct. The looting trigger is
  `MayPayThenEffect { cost: Cost::DiscardCard, then: DrawCards(1) }`, which preserves the printed
  optionality and the "if you do" conditionality — i.e. the exact defect Smuggler's Copter has, and
  this def does **not** have it. Only the auto-choice of which card to discard is engine-made.

### Hullbreaker Horror — **CLASS B**
- declared completeness: `Complete` (BY `#[default]` — no field declared)
- MCP printed oracle text: "Flash\nThis spell can't be countered.\nWhenever you cast a spell, choose up to one —\n• Return target spell you don't control to its owner's hand.\n• Return target nonland permanent to its owner's hand."
- stored `oracle_text` field: matches
- mana cost `{5}{U}{U}` ✓, type `Creature — Kraken Horror` ✓, P/T 7/8 ✓, Flash ✓,
  `cant_be_countered: true` ✓ (:29)
- verdict rationale: `min_modes: 0 / max_modes: 1` is the correct encoding of "choose up to one";
  both modes bounce to `PlayerTarget::OwnerOf(...)` — **owner**, not controller, matching printed
  "its owner's hand" and correct in multiplayer gain-control scenarios. Mode 0's target filter is
  `controller: TargetController::Opponent`, the right reading of "a spell you don't control"; mode
  1's is `non_land: true`, the right reading of "nonland permanent" (KI-1 clean). Which mode gets
  chosen is an auto-choice = BASELINE premise.
- **WATCH**: the def declares a **flat** `targets: vec![TargetSpellWithFilter, TargetPermanentWithFilter]`
  (hullbreaker_horror.rs:46-57) with `mode_targets: None` (:85), while Izzet Charm in this same
  batch uses the per-mode `mode_targets` mechanism that PB-AC4/PB-EF7 introduced for exactly this
  shape (CR 700.2c/700.2f: targets are declared only for the chosen mode). The activated-ability
  path hard-rejects mixing the two (`crates/engine/src/rules/abilities.rs:469-476`); I did not
  trace the triggered-ability path far enough to say what a flat two-requirement list does there
  when only one mode is chosen. The def's own comment (":34") concedes the practical consequence —
  *"auto-selects mode 0 (bounce opponent's spell). If no legal target, 0 modes"* — i.e. mode 1 may
  be effectively unreachable whenever there is no opposing spell on the stack, which is most of the
  time. Choosing zero modes **is** a legal answer to "choose up to one", so this stays B; but if
  the flat list makes mode 1 structurally unreachable rather than merely unchosen, that is a D and
  should be re-examined with the triggered-modal-target code in hand.

### Inexorable Tide — **CLASS B**
- declared completeness: `Complete` (BY `#[default]` — no field declared)
- MCP printed oracle text: "Whenever you cast a spell, proliferate. (Choose any number of permanents and/or players, then give each another counter of each kind already there.)"
- stored `oracle_text` field: **DIFFERS, reminder text only** — "Whenever you cast a spell, proliferate. (**You choose** any number of permanents and/or players, then give each another counter of each kind already there.)"
- mana cost `{3}{U}{U}` ✓, type `Enchantment` ✓
- verdict rationale: `WheneverYouCastSpell` with every filter off (`spell_type_filter: None`,
  `noncreature_only: false`, `during_opponent_turn: false` — and `false` there means *no*
  restriction, per the field doc at `card_definition.rs:3361-3366`) is exactly "whenever you cast a
  spell"; effect is `Effect::Proliferate`. Which permanents/players get proliferated is the
  auto-choice.
- **WATCH**: the reminder-text wording drift ("You choose" vs "Choose") is inside the parenthetical
  reminder, carries no rules content (CR 207.2), and changes nothing about play. Recorded for a
  text-hygiene sweep, deliberately **not** counted as class D — inflating D with reminder-text
  typos would bury the two real findings.

### Izzet Charm — **CLASS B**
- declared completeness: `Complete` (BY `#[default]` — no field declared)
- MCP printed oracle text: "Choose one —\n• Counter target noncreature spell unless its controller pays {2}.\n• Izzet Charm deals 2 damage to target creature.\n• Draw two cards, then discard two cards."
- stored `oracle_text` field: matches
- mana cost `{U}{R}` ✓, type `Instant` ✓
- verdict rationale: `min_modes: 1, max_modes: 1` = "choose one"; all three modes present and in
  printed order. Mode 0 is `CounterUnlessPays` with `Cost::Mana(generic 2)` and a
  `non_creature: true` spell filter — the "unless its controller pays {2}" is preserved, not
  dropped to a hard counter. Mode 1 is `DealDamage(2)` at `TargetRequirement::TargetCreature`.
  Mode 2 is `Sequence(DrawCards 2, DiscardCards 2)` — mandatory in the def and **mandatory on the
  printed card** ("Draw two cards, then discard two cards" has no "may"), so this is *not* the
  Smuggler's Copter shape. Targets are correctly per-mode via `mode_targets` (CR 700.2c). Only the
  mode choice, the discard choice and the spell controller's pay/decline are engine-made.

### Kalastria Highborn — **CLASS B**
- declared completeness: `Complete` (**explicit**, kalastria_highborn.rs:59)
- MCP printed oracle text: "Whenever this creature or another Vampire you control dies, you may pay {B}. If you do, target player loses 2 life and you gain 2 life."
- stored `oracle_text` field: **DIFFERS, self-reference form only** — "Whenever **Kalastria Highborn** or another Vampire you control dies, ..." (the pre-2021 templating of the identical clause)
- mana cost `{B}{B}` ✓, type `Creature — Vampire Shaman` ✓, P/T 2/2 ✓
- verdict rationale: `WheneverCreatureDies { controller: Some(You), exclude_self: false,
  filter: has_subtype Vampire }` covers both halves of "this creature or another Vampire you
  control" in one condition, correctly — Kalastria is herself a Vampire you control, so
  `exclude_self: false` is required, and the def's inline comment says so. The `MayPayThenEffect`
  preserves the optional {B} and the "if you do" conditionality. `LoseLife` goes to
  `DeclaredTarget { index: 0 }` (printed "target player", with `TargetRequirement::TargetPlayer`
  declared) and `GainLife` goes to `PlayerTarget::Controller` (printed "**you** gain 2 life") —
  the two recipients are correctly distinct, which is the classic multiplayer trap and this def
  avoids it.
- **WATCH**: stored `oracle_text` uses the old "Whenever Kalastria Highborn" self-reference where
  current Oracle reads "Whenever this creature". Cosmetic; no rules content differs. Text-hygiene
  sweep item, not a D.

### Karn's Bastion — **CLASS B**
- declared completeness: `Complete` (BY `#[default]` — no field declared)
- MCP printed oracle text: "{T}: Add {C}.\n{4}, {T}: Proliferate. (Choose any number of permanents and/or players, then give each another counter of each kind already there.)"
- stored `oracle_text` field: matches
- mana cost: `None` ✓ (land), type `Land` ✓ — **no** supertype, and MCP agrees (plain `Land`)
- verdict rationale: both abilities present with the right costs — `Cost::Tap` → `mana_pool(0,0,0,0,0,1)`
  (WUBRGC order: one **colorless**, correct for `{C}`), and `Cost::Sequence([Mana{generic:4}, Tap])`
  → `Effect::Proliferate`. ETB cross-check (check 11): the printed card has **no** enters-tapped
  clause and the def has **no** ETB-tapped replacement — consistent in both directions. Only the
  proliferate selection is auto-chosen.

### Kindred Dominance — **CLASS B**
- declared completeness: `Complete` (BY `#[default]` — no field declared)
- MCP printed oracle text: "Choose a creature type. Destroy all creatures that aren't of the chosen type."
- stored `oracle_text` field: matches
- mana cost `{5}{B}{B}` ✓, type `Sorcery` ✓
- verdict rationale: `Sequence(ChooseCreatureType { default: Human }, DestroyAll { filter: creature
  + exclude_chosen_subtype: true, cant_be_regenerated: false })`. The ordering is right (the choice
  is made on resolution, before the destruction), the filter negation is right ("that **aren't** of
  the chosen type"), and `cant_be_regenerated: false` is right — the printed card says only
  "Destroy", with no "can't be regenerated" clause. The `default: Human` is precisely the
  engine-made choice the BASELINE records; the printed card gives the choice to the controller, and
  nothing else about the def deviates.

### Korvold, Fae-Cursed King — **CLASS B**
- declared completeness: `Complete` (BY `#[default]` — no field declared)
- MCP printed oracle text: "Flying\nWhenever Korvold enters or attacks, sacrifice another permanent.\nWhenever you sacrifice a permanent, put a +1/+1 counter on Korvold and draw a card."
- stored `oracle_text` field: **DIFFERS, name form only** — "Whenever **Korvold, Fae-Cursed King** enters or attacks, ..." (full name where current Oracle uses the short name)
- mana cost `{2}{B}{R}{G}` ✓, type `Legendary Creature — Dragon Noble` ✓ (`SuperType::Legendary` at :22), P/T 4/4 ✓, Flying ✓
- verdict rationale: the "enters or attacks" clause is split into two triggers
  (`WhenEntersBattlefield` + `WhenAttacks`), each an exact translation of one half — a structural
  decomposition, not a semantic change. Both carry `exclude_self: true` for printed "**another**
  permanent" (the "another"-exclusion check, correct) and `controller: TargetController::You`
  (you sacrifice your own permanent, correct). The sacrifice is **forced**, matching printed —
  there is no "may". The reward trigger is `WheneverYouSacrifice { filter: None,
  player_filter: None }`; `player_filter: None` means controller-only per the field doc
  (`card_definition.rs:3550-3553`), which is the right reading of "whenever **you** sacrifice".
  Counter goes on `EffectTarget::Source` (Korvold himself) and the draw to `Controller`. Only
  *which* permanent is sacrificed is engine-made.
- **WATCH**: stored `oracle_text` uses the full name where current Oracle uses "Korvold".
  Cosmetic; text-hygiene sweep item, not a D.

### Leaf-Crowned Visionary — **CLASS B**
- declared completeness: `Complete` (BY `#[default]` — no field declared)
- MCP printed oracle text: "Other Elves you control get +1/+1.\nWhenever you cast an Elf spell, you may pay {G}. If you do, draw a card."
- stored `oracle_text` field: matches
- mana cost `{G}{G}` ✓, type `Creature — Elf Druid` ✓, P/T 1/1 ✓
- verdict rationale: the anthem is `Static` at `EffectLayer::PtModify` with
  `EffectFilter::OtherCreaturesYouControlWithSubtype(Elf)` — the "**Other**" exclusion is in the
  filter name, so Leaf-Crowned Visionary does not pump itself, matching printed. Duration is
  `WhileSourceOnBattlefield`, correct for a static ability. The trigger uses
  `spell_subtype_filter: Some([Elf])` for "an Elf spell" (CR 205.1a — the *spell's* subtype, which
  is the right axis; an Elf spell need not be a creature spell) and `MayPayThenEffect` preserves
  both the optional {G} and the "if you do". No auto-choice beyond the pay/decline.

---

SUMMARY batch4: 12 class-B, 2 class-D (Grateful Apparition, Grisly Salvage), 5 watch (Hullbreaker Horror — flat `targets` with `mode_targets: None` may make mode 1 unreachable; Inexorable Tide, Kalastria Highborn, Korvold — cosmetic stored-`oracle_text` drift; Grateful Apparition — no planeswalker-damage `TriggerCondition` variant appears to exist, so the fix may be a `partial` marker rather than a def repair)

Additional standing note for the PB-DX4 rollup: **12 of these 14 defs declare no `completeness`
field at all** and are `Complete` only by the `#[default]` derive. The two class-D defs are both in
that group. Explicit markers appear on only Growing Rites of Itlimoc and Kalastria Highborn.
