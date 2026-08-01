# PB-DX4 — class-B/class-D triage of the PB-DP10 `BASELINE`, batch 5 of 7

**Date**: 2026-08-01
**Scope**: 14 `Complete` defs frozen in `crates/engine/tests/core/decision_gate.rs::BASELINE`.
**Method**: MCP `lookup_card` (printed oracle text / type line / mana cost / P-T / keywords) read
against the full def file, clause by clause. Read-only; no file was edited.
**Reminder**: the engine auto-picking among legal options (which permanent to sacrifice, which card
to discard, which creature type, whether the countered spell's controller pays) is the *premise* of
the BASELINE, not a defect. Class D is reserved for the def being wrong against printed text.

---

### Make Disappear — **CLASS B**
- declared completeness: `Complete` (BY `#[default]` — no `completeness` field declared)
- MCP printed oracle text: "Casualty 1 (As you cast this spell, you may sacrifice a creature with power 1 or greater. When you do, copy this spell and you may choose a new target for the copy.)\nCounter target spell unless its controller pays {2}."
- stored `oracle_text` field: matches (verbatim, including the reminder text)
- type line `Instant`, mana cost `{1}{U}` (`generic: 1, blue: 1`) — both match.
- verdict rationale: both printed clauses are present — `KeywordAbility::Casualty(1)` for the
  casualty clause and `Effect::CounterUnlessPays { cost: {2} }` targeting
  `TargetSpellWithFilter(default)` for the counter clause. The BASELINE row
  (`counter_unless_pays`) is exactly the auto-choice the premise permits: the engine decides for
  the spell's controller whether to pay the {2}. Nothing is printed that the def omits.
- WATCH: casualty is carried by the keyword marker alone (no companion `AbilityDefinition` for the
  sacrifice/copy), and Make Disappear is the corpus's **only** `Casualty` def, so there is no
  sibling to compare the convention against. Whether the engine's `Casualty` handling actually
  offers the sacrifice and copies the spell is an *engine* question outside this triage; the def
  faithfully declares the printed keyword either way. Not a class-D defect.

---

### Mana Leak — **CLASS B**
- declared completeness: `Complete` (BY `#[default]` — no `completeness` field declared)
- MCP printed oracle text: "Counter target spell unless its controller pays {3}."
- stored `oracle_text` field: matches (verbatim)
- type line `Instant`, mana cost `{1}{U}` — both match. No P/T (correct for an instant).
- verdict rationale: single printed clause, single `Effect::CounterUnlessPays` with
  `cost: Cost::Mana({3})` on `TargetSpellWithFilter(default)` ("target spell", unrestricted).
  The auto-choice is the controller's pay/decline, which is the BASELINE premise.

---

### Mana Tithe — **CLASS B**
- declared completeness: `Complete` (BY `#[default]` — no `completeness` field declared)
- MCP printed oracle text: "Counter target spell unless its controller pays {1}."
- stored `oracle_text` field: matches (verbatim)
- type line `Instant`, mana cost `{W}` (`white: 1`, no generic) — both match.
- verdict rationale: identical shape to Mana Leak with the correct `{1}` amount. Faithful.

---

### Merciless Executioner — **CLASS B**
- declared completeness: `Complete` (BY `#[default]` — no `completeness` field declared)
- MCP printed oracle text: "When this creature enters, each player sacrifices a creature of their choice."
- stored `oracle_text` field: DIFFERS (wording only) — "When this enters, each player sacrifices a creature."
- type line `Creature — Orc Warrior`, mana cost `{2}{B}`, P/T `3/1` — all three match.
- verdict rationale: the ability is right in every mechanically load-bearing respect —
  `WhenEntersBattlefield` → `SacrificePermanents { player: EachPlayer, count: Fixed(1),
  filter: has_card_type Creature }`. Printed says **each player** (not each opponent) and the def
  uses `PlayerTarget::EachPlayer`, which is correct and includes the controller. The BASELINE row
  `sacrifice_permanents` is precisely the "of their choice" the engine currently makes for each
  player — the premise, not a defect.
- WATCH: the stored `oracle_text` is a pre-2024-templating rendering. It drops the phrase "of their
  choice" and says "When this enters" where the current printing says "When this creature enters".
  This is *cosmetic drift in a display field*, not a mechanical divergence (unlike Shambling Ghast,
  whose stored text named a different trigger event from the one authored), so I am not calling it
  class D. Worth a corpus-wide re-render of `oracle_text` against MCP as a separate cheap sweep.

---

### Metastatic Evangel — **CLASS D**
- declared completeness: `Complete` (BY `#[default]` — no `completeness` field declared)
- MCP printed oracle text: "Whenever another nontoken creature you control enters, proliferate. (Choose any number of permanents and/or players, then give each another counter of each kind already there.)"
- MCP printed type line: "Creature — Phyrexian Human Cleric"; MCP printed mana cost: "{1}{W}"; MCP printed P/T: "3/1"
- stored `oracle_text` field: matches (verbatim, including reminder text)
- verdict rationale: the *ability* is authored correctly (the trigger, the `controller: You` filter,
  `exclude_self: true` for "another", and `Effect::Proliferate`), and the BASELINE row
  `proliferate` is a legitimate auto-choice. But the card's **printed characteristics are wrong in
  three independent ways** — mana cost, power/toughness (inverted) and a missing printed subtype —
  none of which has anything to do with the auto-choice. Verified twice against MCP (second lookup
  with rulings returned identical values).
- DEFECT 1: printed mana cost is **`{1}{W}`** vs def
  `mana_cost: Some(ManaCost { generic: 2, white: 1, .. })` = `{2}{W}` at
  `crates/card-defs/src/defs/metastatic_evangel.rs:10-14` (and the header comment at :1 repeats the
  wrong cost). Effect on play: the card costs one generic mana too much to cast, every cast, in
  every game; it also mis-reports converted mana cost / mana value to every effect that reads it.
- DEFECT 2: printed P/T is **`3/1`** vs def `power: Some(1), toughness: Some(3)` at
  `crates/card-defs/src/defs/metastatic_evangel.rs:20-21` — the two are **transposed**. Effect on
  play: a 1/3 body instead of a 3/1 — it deals 2 less combat damage, survives damage it should die
  to, and dies to damage it should survive; every combat involving it resolves wrongly.
- DEFECT 3: printed type line is "Creature — Phyrexian **Human** Cleric" vs def
  `types: creature_types(&["Phyrexian", "Cleric"])` at
  `crates/card-defs/src/defs/metastatic_evangel.rs:15` — the **Human** subtype is absent. Effect on
  play: Human tribal anthems / lords / "choose a creature type" effects (including Obelisk of Urd
  and Patchwork Banner in this very batch) fail to see it; Human-typed removal and tutors miss it.
- DEFECT 4 (secondary, self-documented): printed "another **nontoken** creature you control enters"
  vs the def's trigger, which carries no token axis at all —
  `TriggerCondition::WheneverCreatureEntersBattlefield { filter: Some(TargetFilter { controller:
  You, .. }), exclude_self: true }` at :33-39. The def's own comment at :26-30 states that
  `is_token` in `TargetFilter` is ignored on the ETB-trigger path and that "a token creature ETB
  would still fire this trigger today". Effect on play: the trigger over-fires — every token
  creature entering under your control proliferates when the printed card says it must not.
  Recorded as a defect rather than a DSL-gap excuse because a `Complete` marker asserts the printed
  card is faithfully encoded; the acknowledgement is in a comment, not in the completeness field.

---

### Miara, Thorn of the Glade — **CLASS B**
- declared completeness: `Complete` (BY `#[default]` — no `completeness` field declared)
- MCP printed oracle text: "Whenever Miara or another Elf you control dies, you may pay {1} and 1 life. If you do, draw a card.\nPartner (You can have two commanders if both have partner.)"
- stored `oracle_text` field: matches in substance — the def expands the self-reference to the full
  name ("Whenever Miara, Thorn of the Glade or another Elf you control dies…"), which is the older
  full-name templating of the identical clause. Not a mechanical divergence.
- type line `Legendary Creature — Elf Scout` (`full_types` with `SuperType::Legendary` present),
  mana cost `{1}{B}`, P/T `1/2` — all match, supertype correctly declared.
- verdict rationale: `WheneverCreatureDies { controller: You, exclude_self: false, filter:
  has_subtype Elf }` correctly covers "Miara **or** another Elf you control" in one predicate,
  because Miara is herself an Elf and `exclude_self: false` keeps her in scope — the def's own
  comment at :29-32 states this reasoning and it holds. The printed optionality is preserved:
  `Effect::MayPayThenEffect` with `Cost::Sequence([Mana {1}, PayLife(1)])` gates the draw, so the
  "you may pay … If you do" is *not* dropped (this is the opposite of the Smuggler's Copter shape).
  `Partner` keyword present. The auto-choice is only whether to pay.

---

### Misdirection — **CLASS B**
- declared completeness: `Complete` (BY `#[default]` — no `completeness` field declared)
- MCP printed oracle text: "You may exile a blue card from your hand rather than pay this spell's mana cost.\nChange the target of target spell with a single target."
- stored `oracle_text` field: matches (verbatim)
- type line `Instant`, mana cost `{3}{U}{U}` (`generic: 3, blue: 2`) — both match.
- verdict rationale: both printed clauses present and correctly scoped. The alternative cost is
  `AltCastAbility { kind: Pitch, cost: ManaCost::default(), details: Pitch { costs:
  [ExileFromHand { color: Blue }], opponents_turn_only: false } }` — exile a **blue** card, no life
  component (correctly distinguished from Force of Will in the def's comment), and no
  opponent's-turn restriction, matching the printed text. The effect is `ChangeTargets` on
  `TargetSpellWithSingleTarget`, spell-only per the printed "target spell" (not "spell or
  ability"); the def argues this explicitly at :7-10 and the argument is right.
- WATCH: `must_change: true`. CR 115.7b requires changing the target if a legal alternative exists
  and leaves it unchanged if none does; "must" is the correct printed reading, so this is only a
  WATCH on whether the engine's `must_change` degrades gracefully to "no legal change" rather than
  fizzling. Def-side text is faithful.

---

### Morophon, the Boundless — **CLASS B**
- declared completeness: `Complete` (BY `#[default]` — no `completeness` field declared)
- MCP printed oracle text: "Changeling (This card is every creature type.)\nAs Morophon enters, choose a creature type.\nSpells of the chosen type you cast cost {W}{U}{B}{R}{G} less to cast. This effect reduces only the amount of colored mana you pay.\nOther creatures you control of the chosen type get +1/+1."
- stored `oracle_text` field: matches (verbatim)
- type line `Legendary Creature — Shapeshifter` (`SuperType::Legendary` present), mana cost `{7}`,
  P/T `6/6` — all match.
- verdict rationale: all four printed lines are encoded, and the two easy-to-get-wrong details are
  both right. (a) The anthem uses `EffectFilter::OtherCreaturesYouControlOfChosenType` — printed
  says "**Other** creatures", and the *Other* variant is the one selected, so Morophon does not pump
  herself. (b) The cost reduction is `colored_mana_reduction: Some({W}{U}{B}{R}{G} one each)` with
  `change: 0` generic, which is exactly the printed "reduces only the amount of **colored** mana you
  pay"; `scope: Controller` matches "you cast". The "As … enters, choose a creature type"
  self-replacement is a `Replacement` with `is_self: true` (CR 614.1c), which is the right shape —
  the choice must be made before the permanent is on the battlefield or the anthem would have a
  one-window gap. The `SubType("Human")` inside `ChooseCreatureType` is the engine's auto-pick
  default, i.e. the BASELINE row itself, not a printed-text defect.

---

### Nadir Kraken — **CLASS B**
- declared completeness: `Complete` (BY `#[default]` — no `completeness` field declared)
- MCP printed oracle text: "Whenever you draw a card, you may pay {1}. If you do, put a +1/+1 counter on this creature and create a 1/1 blue Tentacle creature token."
- stored `oracle_text` field: matches in substance — the def uses the older self-reference
  ("…put a +1/+1 counter on Nadir Kraken…") where the current printing says "on this creature".
  Same referent, no mechanical difference.
- type line `Creature — Kraken`, mana cost `{1}{U}{U}` (`generic: 1, blue: 2`), P/T `2/3` — all match.
- verdict rationale: `WheneverYouDrawACard` → `MayPayThenEffect { cost: {1} }` wrapping a
  `Sequence([AddCounter { target: Source, PlusOnePlusOne, 1 }, CreateToken])`. The printed
  optionality is **preserved**, and both halves of the "if you do" are correctly *inside* the gated
  branch rather than unconditional. Token spec is 1/1, blue, Creature — Tentacle, count 1,
  untapped, no keywords — matching printed exactly. The BASELINE row `may_pay_then_effect` is the
  engine deciding whether to pay, which is the premise.

---

### Nether Traitor — **CLASS B**
- declared completeness: **`Complete` (EXPLICIT)** — `completeness: Completeness::Complete` at
  `crates/card-defs/src/defs/nether_traitor.rs:60`. (The only def in this batch that declares one.)
- MCP printed oracle text: "Haste\nShadow (This creature can block or be blocked by only creatures with shadow.)\nWhenever another creature is put into your graveyard from the battlefield, you may pay {B}. If you do, return this card from your graveyard to the battlefield."
- stored `oracle_text` field: matches (verbatim)
- type line `Creature — Spirit`, mana cost `{B}{B}` (`black: 2`, no generic), P/T `1/1` — all match.
- verdict rationale: all three printed lines present — `Keyword(Haste)`, `Keyword(Shadow)`, and a
  `trigger_zone: Some(TriggerZone::Graveyard)` triggered ability (correct: the ability must
  function from the graveyard). Optionality preserved via `MayPayThenEffect { cost: {B} }` gating
  the `MoveZone` to battlefield, untapped, no controller override. `exclude_self: true` encodes the
  printed "**another** creature". `nontoken_only: false` is right — a token creature put into the
  graveyard does trigger before it ceases to exist.
- WATCH: printed "put into **your** graveyard" is an *ownership* condition (CR 404.3), and the def
  approximates it with `controller: Some(TargetController::You)`. The def documents this at :30-34.
  The two diverge only under gain-control effects (a creature you own but an opponent controls dies
  → printed fires, def does not; and the mirror case). I am classifying B rather than D because the
  divergence requires a control-change effect to be observable and the def states the approximation
  explicitly with the corpus convention (athreos, fecundity) behind it — but it *is* a live
  owner-vs-controller multiplayer divergence on a `Complete`-marked card and is the strongest
  WATCH in this batch.

---

### Obelisk of Urd — **CLASS B**
- declared completeness: `Complete` (BY `#[default]` — no `completeness` field declared)
- MCP printed oracle text: "Convoke (Your creatures can help cast this spell. Each creature you tap while casting this spell pays for {1} or one mana of that creature's color.)\nAs this artifact enters, choose a creature type.\nCreatures you control of the chosen type get +2/+2."
- stored `oracle_text` field: matches (verbatim)
- type line `Artifact`, mana cost `{6}` — both match. No P/T (correct, non-creature artifact).
- verdict rationale: all three printed lines encoded — `Keyword(Convoke)`, the `is_self: true`
  `Replacement` carrying `ChooseCreatureType` (correct CR 614.1c shape, and the def's comment at
  :24-28 gives the right reason: a `Triggered` form would leave a window where
  `chosen_creature_type` is `None` while the permanent is already on the battlefield), and the
  Layer 7c anthem with `LayerModification::ModifyBoth(2)` — **+2/+2**, the right amount — over
  `EffectFilter::CreaturesYouControlOfChosenType`. Printed says "Creatures you control" with no
  "other", and the *non*-Other filter variant is used, which is correct (Obelisk is not a creature
  anyway). `SubType("Human")` is the engine's auto-pick default = the BASELINE row.

---

### Pact of the Serpent — **CLASS B**
- declared completeness: `Complete` (BY `#[default]` — no `completeness` field declared)
- MCP printed oracle text: "Choose a creature type. Target player draws X cards and loses X life, where X is the number of creatures they control of the chosen type."
- stored `oracle_text` field: matches (verbatim)
- type line `Sorcery`, mana cost `{1}{B}{B}` (`generic: 1, black: 2`) — both match.
- verdict rationale: this is the multiplayer-correctness case in the batch and it is authored
  **right**. Printed says "**Target player** draws X and loses X, where X is the number of
  creatures **they** control" — all three player references must be the target, not the caster, and
  all three are: `DrawCards { player: DeclaredTarget { index: 0 } }`, `LoseLife { player:
  DeclaredTarget { index: 0 } }`, and both `EffectAmount::ChosenTypeCreatureCount { controller:
  DeclaredTarget { index: 0 } }`. `LoseLife` (not `DealDamage`) matches "loses X life". The
  `ChooseCreatureType` is sequenced *first*, inside the resolution, matching the 2021-02-05 ruling
  quoted in the def header that the type is chosen on resolution rather than on cast. The BASELINE
  row `choose_color_or_type` is that auto-picked type.

---

### Patchwork Banner — **CLASS B**
- declared completeness: `Complete` (BY `#[default]` — no `completeness` field declared; the
  comment at :53 says so explicitly — "PB-EF12 (EF-W-PB2-3): un-marked, see birds_of_paradise.rs
  for the fix")
- MCP printed oracle text: "As this artifact enters, choose a creature type.\nCreatures you control of the chosen type get +1/+1.\n{T}: Add one mana of any color."
- stored `oracle_text` field: matches (verbatim)
- type line `Artifact`, mana cost `{3}` — both match. No P/T (correct).
- verdict rationale: all three printed lines present — the `is_self: true` `ChooseCreatureType`
  replacement, the Layer 7c `ModifyBoth(1)` (**+1/+1**, correct — not Obelisk's +2/+2) over
  `CreaturesYouControlOfChosenType`, and `Activated { cost: Cost::Tap, effect: AddManaAnyColor }`.
  Printed mana ability has **no** creature-type restriction on the mana (unlike the "of the chosen
  type" restriction on the anthem) and the def correctly imposes none.
- WATCH: the in-def comment flags this def as deliberately un-marked pending a fix that never
  landed; per the `#[default] Completeness::Complete` derive it is nonetheless deck-legal today.
  That is a marker-hygiene observation, not an oracle defect — the def is faithful.

---

### Pull from Tomorrow — **CLASS B**
- declared completeness: `Complete` (BY `#[default]` — no `completeness` field declared)
- MCP printed oracle text: "Draw X cards, then discard a card."
- stored `oracle_text` field: matches (verbatim)
- type line `Instant`, mana cost `{X}{U}{U}` (`x_count: 1, blue: 2`) — both match.
- verdict rationale: `Sequence([DrawCards { Controller, XValue }, DiscardCards { Controller,
  Fixed(1) }])`. Critically, this is **not** the Smuggler's Copter shape even though the DSL is a
  bare `Sequence`: the printed text here is genuinely mandatory ("Draw X cards, **then** discard a
  card" — no "may", no "if you do"), so an unconditional sequence is the correct encoding and the
  `then` ordering is preserved. Both halves correctly target the caster (`Controller`), matching
  the printed absence of any other player reference. The BASELINE row `discard_cards` is the engine
  picking *which* card to discard — the premise, not a defect.

---

## Cross-cutting observations

1. **13 of 14 defs in this batch declare no `completeness` field at all** and are `Complete` only
   by the `#[default]` derive. Only `nether_traitor.rs` declares one explicitly. This is the same
   silent-defect generator that produced `aurelia_the_warleader` (PB-DX1) and
   `emeria_the_sky_ruin` (PB-DX3b), and Metastatic Evangel is a third instance: a def with three
   wrong printed characteristics is deck-legal because nobody ever wrote a marker. The corpus-wide
   question "which defs never declare a marker?" is still unasked and is cheap.
2. **Stored `oracle_text` drifts from current templating in three defs** (Merciless Executioner,
   Miara, Nadir Kraken) — all in the harmless direction (old full-name self-references, missing "of
   their choice"). None changes behaviour, so none is class D, but a mechanical re-render of
   `oracle_text` against MCP across the corpus would remove a whole category of future
   false-positive triage work.
3. **The "may" test that defined class D was passed by every def in this batch that needed it.**
   Miara, Nadir Kraken and Nether Traitor all use `Effect::MayPayThenEffect` and all three keep the
   entire "if you do" payload inside the gated branch. Pull from Tomorrow's bare `Sequence` is
   correct precisely because its printed text has no "may". The Smuggler's Copter failure mode does
   not recur here.

---

SUMMARY batch5: 13 class-B, 1 class-D (Metastatic Evangel), 6 watch
