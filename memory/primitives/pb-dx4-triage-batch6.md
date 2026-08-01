# PB-DX4 — class-B/class-D triage, batch 6 of 7

**Date**: 2026-08-01
**Scope**: 14 `BASELINE` members from `crates/engine/tests/core/decision_gate.rs` (rows as frozen there).
**Method**: MCP `lookup_card` (printed oracle text / type line / mana cost / P-T / keywords) read as
authoritative, then the def file read in full and compared clause by clause. Read-only; no file edited.

Convention used below: "declared completeness" distinguishes an **explicit** `completeness:
Completeness::Complete` field from a def that ends in `..Default::default()` and is therefore
`Complete` **by the `#[default]` derive with no field declared at all** — the silent-defect generator
that has already bitten this project twice (`aurelia_the_warleader`, `emeria_the_sky_ruin`).

---

### Radstorm — **CLASS D**
- BASELINE row: `proliferate`
- declared completeness: `Complete` (BY `#[default]` — no field declared; def ends `..Default::default()`)
- MCP printed mana cost: **`{3}{U}`**; type `Instant`; keywords `["Storm","Proliferate"]`
- MCP printed oracle text: "Storm (When you cast this spell, copy it for each spell cast before it this turn.)\nProliferate. (Choose any number of permanents and/or players, then give each another counter of each kind already there.)"
- stored `oracle_text` field: **DIFFERS** — "Storm (When you cast this spell, copy it for each spell cast before it this turn. **You may choose new targets for the copies.**)\nProliferate. (Choose any number of permanents and/or players, then give each another counter of each kind already there.)"
- verdict rationale: the ability content (Storm keyword + `Effect::Proliferate` spell) is faithful and the
  proliferate auto-choice is exactly the expected class-B deviation — but the **mana cost is one generic
  short of the printed card**, which is a plain def error independent of any decision.
- DEFECT 1: printed mana cost **`{3}{U}`** vs def `mana_cost: Some(ManaCost { generic: 2, blue: 1, .. })`
  at `crates/card-defs/src/defs/radstorm.rs:11-15` (and the same error is repeated in the file's
  header comment at line 1, `// Radstorm — {2}{U}, Instant`).
  Effect on play: the spell is castable for one mana less than printed, and — because this is a **Storm**
  card whose whole point is casting many spells in a turn — the discount compounds directly into extra
  copies. It also mis-values every "spells with mana value N" and cost-reduction interaction.
- DEFECT 2 (minor, same card): printed Storm reminder text is "(When you cast this spell, copy it for
  each spell cast before it this turn.)" vs the stored `oracle_text`'s "...before it this turn. **You may
  choose new targets for the copies.**)" at `radstorm.rs:17-21`. Effect on play: none mechanically
  (reminder text is not executed), but the stored field no longer reproduces the printed card, which is
  the DP10-8 class of defect this triage exists to record.
- WATCH: none beyond the above.

---

### Raffine's Informant — **CLASS B**
- BASELINE row: `connive`
- declared completeness: `Complete` (explicit, `raffines_informant.rs:47`)
- MCP printed: `{1}{W}`, `Creature — Human Wizard`, 2/1, keywords `["Connive"]`
- MCP printed oracle text: "When this creature enters, it connives. (Draw a card, then discard a card. If you discarded a nonland card, put a +1/+1 counter on this creature.)"
- stored `oracle_text` field: **templating drift only** — "When Raffine's Informant enters the
  battlefield, it connives. (Draw a card, then discard a card. If you discarded a nonland card, put a
  +1/+1 counter on this creature.)" (pre-2024 "<name> enters the battlefield" wording for the current
  "this creature enters"; semantically identical).
- verdict rationale: mana cost, type line, P/T and the single ETB-connive trigger all match. `Effect::Connive
  { target: Source, count: Fixed(1) }` is Connive 1 on itself, which is the printed ability verbatim. The
  only runtime deviation is that the engine picks the discarded card (documented fallback: first card in
  hand, alphabetical) instead of asking — the class-B premise exactly.
- WATCH: the `oracle_text` field uses pre-2024 templating. Corpus-wide cosmetic drift, not scored as a
  defect here (see the batch note at the end); flagging so a later sweep can decide policy once.

---

### Raiders' Wake — **CLASS B**
- BASELINE row: `discard_cards`
- declared completeness: `Complete` (BY `#[default]` — no field declared)
- MCP printed: `{3}{B}`, `Enchantment`, keywords `["Raid"]`
- MCP printed oracle text: "Whenever an opponent discards a card, that player loses 2 life.\nRaid — At the beginning of your end step, if you attacked this turn, target opponent discards a card."
- stored `oracle_text` field: **matches** (verbatim, including the em-dash Raid label).
- verdict rationale: both abilities present and correct. Ability 1: `WheneverOpponentDiscards` →
  `LoseLife { player: TriggeringPlayer, amount: 2 }` — "**that player** loses 2 life" correctly routes to
  the discarding opponent, not the controller. Ability 2: `AtBeginningOfYourEndStep` with
  `intervening_if: Some(Condition::YouAttackedThisTurn)` (CR 603.4 raid gate, present) and
  `targets: vec![TargetRequirement::TargetOpponent]` with the discard aimed at
  `DeclaredTarget { index: 0 }` — correct player, correct count. The only auto-choice is which card the
  targeted opponent discards, which is the BASELINE row.
- WATCH: none.

---

### Retreat to Kazandu — **CLASS B**
- BASELINE row: `modal_trigger`
- declared completeness: `Complete` (BY `#[default]` — no field declared)
- MCP printed: `{2}{G}`, `Enchantment`, keywords `["Landfall"]`
- MCP printed oracle text: "Landfall — Whenever a land you control enters, choose one —\n• Put a +1/+1 counter on target creature.\n• You gain 2 life."
- stored `oracle_text` field: **matches** (verbatim).
- verdict rationale: trigger filter is `has_card_type: Land` + `controller: You` — "a land **you control**
  enters" exactly. Mode 0 puts one `PlusOnePlusOne` counter on `DeclaredTarget { index: 0 }` with the
  target declared as unrestricted `TargetCreature` ("target creature", correctly *not* limited to yours);
  mode 1 gains 2 life to `PlayerTarget::Controller` ("**you** gain 2 life"). `min_modes: 1, max_modes: 1`
  matches "choose one". The engine choosing the mode is the recorded auto-choice.
- WATCH: `targets` is declared at the ability level (`retreat_to_kazandu.rs:39-42`) with
  `mode_targets: None`, so the *mode-0* target requirement is attached to the trigger unconditionally.
  On a board with **no creature at all**, printed Retreat to Kazandu can still be put on the stack by
  choosing mode 1 ("you gain 2 life"); the def may not be able to. Not scored as D because it is a
  targeting-plumbing consequence of the same modal machinery the BASELINE row is about, and the
  common case (any creature on any board) is correct. Worth a separate `mode_targets` seed.

---

### Risen Reef — **CLASS B**
- BASELINE row: `look_at_top_or_route`
- declared completeness: `Complete` (BY `#[default]` — no field declared)
- MCP printed: `{1}{G}{U}`, `Creature — Elemental`, 1/1, keywords `[]`
- MCP printed oracle text: "Whenever this creature or another Elemental you control enters, look at the top card of your library. If it's a land card, you may put it onto the battlefield tapped. If you don't put the card onto the battlefield, put it into your hand."
- stored `oracle_text` field: **templating drift only** — "Whenever Risen Reef or another Elemental you control enters, ..." (rest verbatim).
- verdict rationale: mana cost, type, P/T match. Trigger is `WheneverCreatureEntersBattlefield` filtered
  to `has_subtype: Elemental` + `controller: You` with `exclude_self: false`, which is precisely "this
  creature **or another** Elemental you control" (Risen Reef is itself an Elemental you control).
  `count: Fixed(1)` means `RevealAndRoute`'s route-all-matches semantics can move **at most one** card,
  so there is no cardinality error here. Land → `Battlefield { tapped: true }`, non-land → `Hand`, both
  destinations exactly as printed. The printed "**you may** put it onto the battlefield tapped / if you
  don't, put it into your hand" is a genuine binary player choice that the engine resolves by always
  taking the battlefield branch — that is the auto-choice the `look_at_top_or_route` row records.
- WATCH 1: printed says "**look at** the top card"; the def uses `Effect::RevealAndRoute`, whose own doc
  (`card_definition.rs:2020`) says "All revealed cards are visible to all players (CR 701.20a)". That is
  a hidden-information deviation (public reveal where the card grants private information), latent today
  because Invariant 7 filtering lands in M10. Not scored as D — no *game-state* outcome differs.
- WATCH 2: the auto-choice is hardwired to the battlefield branch, so "put it into your hand instead"
  is unreachable for a land. Correct classification is still B (a legal option, auto-picked), but it is
  never the *other* legal option, which makes it a candidate for the first real decision channel.

---

### Roalesk, Apex Hybrid — **CLASS B**
- BASELINE row: `proliferate`
- declared completeness: `Complete` (explicit, `roalesk_apex_hybrid.rs:64`)
- MCP printed: `{2}{G}{G}{U}`, `Legendary Creature — Human Mutant`, 4/5, keywords `["Flying","Trample","Proliferate"]`
- MCP printed oracle text: "Flying, trample\nWhen Roalesk enters, put two +1/+1 counters on another target creature you control.\nWhen Roalesk dies, proliferate, then proliferate again."
- stored `oracle_text` field: **name-form drift only** — "When Roalesk, **Apex Hybrid** enters, ..." /
  "When Roalesk, **Apex Hybrid** dies, ..." where the printed card uses the short name "Roalesk".
- verdict rationale: `full_types(&[SuperType::Legendary], ...)` — the **Legendary supertype is present**
  (KI-4 clean, and specifically checked given the `emeria_the_sky_ruin` spurious-supertype finding two
  batches ago). Mana cost `{2}{G}{G}{U}` and 4/5 match. Flying + Trample keywords present. ETB puts
  `count: 2` `PlusOnePlusOne` counters on a target filtered `controller: You, exclude_self: true` —
  "**another** target creature **you control**", both restrictions encoded. Death trigger is
  `Sequence(vec![Proliferate, Proliferate])` — "proliferate, **then proliferate again**", two separate
  proliferate actions, correct. Only the proliferate choice-set is auto-made.
- WATCH: none.

---

### Roiling Regrowth — **CLASS B**
- BASELINE row: `sacrifice_permanents`
- declared completeness: `Complete` (BY `#[default]` — no field declared)
- MCP printed: `{2}{G}`, `Instant`, keywords `[]`
- MCP printed oracle text: "Sacrifice a land. Search your library for up to two basic land cards, put them onto the battlefield tapped, then shuffle."
- stored `oracle_text` field: **matches** (verbatim).
- verdict rationale: the sacrifice is authored as a **resolution-time** `Effect::SacrificePermanents`
  inside the spell's effect sequence, not as an additional cost — which is correct (the printed text has
  no "As an additional cost"; CR 601.2b does not apply), and the def documents that reasoning. Filter is
  `has_card_type: Land` and payer is `Controller`, matching "Sacrifice a land" (yours, one). The searches
  put basics onto the battlefield **tapped** and a trailing `Effect::Shuffle` supplies "then shuffle".
  The auto-choice (which land is sacrificed) is the BASELINE row.
- WATCH: "search for **up to two**" is modelled as two sequential single-card `Effect::SearchLibrary`
  calls (`roiling_regrowth.rs:37-52`) because `SearchLibrary` finds exactly one card (the known
  OOS-DP9-3 gap). Outcome-equivalent for a player who wants two basics and has them; it does *not*
  express finding zero or one by choice, and it is two shuffle-free searches rather than one. Not scored
  as D — this is the documented engine-side cardinality gap, and the def's approximation is the standard
  corpus idiom (`explosive_vegetation` same shape).

---

### Satyr Wayfinder — **CLASS D**
- BASELINE row: `look_at_top_or_route`
- declared completeness: `Complete` (BY `#[default]` — no field declared)
- MCP printed: `{1}{G}`, `Creature — Satyr`, 1/1, keywords `[]`
- MCP printed oracle text: "When this creature enters, reveal the top four cards of your library. You may put **a land card** from among them into your hand. Put **the rest** into your graveyard."
- stored `oracle_text` field: **matches** (verbatim, including the modern "this creature" templating).
- verdict rationale: mana cost, type line, P/T, trigger and the reveal-four all match, and "reveal" is
  the printed word so `RevealAndRoute` is the right family. But the effect routes **every** land among
  the four to hand, where the printed card allows **at most one**. That is a cardinality error, not an
  auto-choice: no sequence of legal player choices puts two land cards into hand off Satyr Wayfinder.
- DEFECT 1: printed "**You may put a land card from among them into your hand.** Put the rest into your
  graveyard." vs def `Effect::RevealAndRoute { count: EffectAmount::Fixed(4), filter: TargetFilter {
  has_card_type: Some(CardType::Land), .. }, matched_dest: Hand, unmatched_dest: Graveyard }` at
  `crates/card-defs/src/defs/satyr_wayfinder.rs:25-38`. `RevealAndRoute`'s implementation partitions the
  top N and moves **all** matched ids to `matched_dest` (`crates/engine/src/effects/mod.rs:5722-5757`,
  loop `for id in &matched_ids`), and its own doc comment states it "routes ALL matches"
  (`crates/card-types/src/cards/card_definition.rs:2041`).
  Effect on play: with 2, 3 or 4 land cards among the top four, the controller draws **all of them** into
  hand instead of one, and those extra lands never reach the graveyard — so the card is both a much
  stronger ramp/card-advantage engine than printed and a much weaker self-mill enabler
  (graveyard-matters decks get fewer cards binned). Both halves are wrong in the same event.
- Note for whoever fixes it: the correct primitive already exists — `Effect::LookAtTopThenPlace` is
  documented as "the **put-≤1 sibling** of `Effect::RevealAndRoute`" with a `rest_to` destination
  (`card_definition.rs:2037-2046`, shipped by PB-OS8). This is a def-side fix, not an engine gap.
- WATCH: the printed "**You may**" (declining to take a land, sending all four to the graveyard) is a
  genuine player choice; once the count is fixed to ≤1 that residual optionality is a legitimate
  class-B auto-choice and should stay in `BASELINE`.

---

### Shambling Ghast — **CLASS D** (independently re-verified; I agree with all three alleged deviations, and add a fourth)
- BASELINE row: `modal_trigger`
- declared completeness: `Complete` (explicit, `shambling_ghast.rs:74`)
- MCP printed: `{B}`, `Creature — Zombie`, 1/1, keywords **`["Treasure"]`** — there is **no `Decayed`**
- MCP printed oracle text: "When this creature **dies**, choose one —\n• Target creature **an opponent controls** gets **-1/-1 until end of turn**.\n• Create a Treasure token. (It's an artifact with "{T}, Sacrifice this token: Add one mana of any color.")"
- stored `oracle_text` field: **DIFFERS materially** — "Decayed (This creature can't block. When it attacks, sacrifice it at end of combat.)\nWhen Shambling Ghast **enters**, create a Treasure token or put a -1/-1 counter on **target creature**."
- verdict rationale: mana cost `{B}`, `Creature — Zombie`, 1/1, the `WhenDies` trigger condition, the
  two modes and the mode-1 `controller: Opponent` target filter are all correct. Everything else listed
  below is wrong. This is unambiguously class D.
- DEFECT 1 (phantom keyword): printed keyword list is **`["Treasure"]`** and the printed card carries no
  Decayed clause at all, vs def `AbilityDefinition::Keyword(KeywordAbility::Decayed)` at
  `crates/card-defs/src/defs/shambling_ghast.rs:24`.
  Effect on play: the creature **can't block** and is **sacrificed at end of combat whenever it attacks**
  (CR 702.146a), neither of which the printed card does. It is the single largest behavioural error in
  the batch: a 1/1 blocker is silently removed from every defensive board, and every attack self-destructs
  the creature — which then *fires the death trigger*, so the def also generates Treasure tokens the
  printed card never would.
- DEFECT 2 (wrong duration): printed "• Target creature an opponent controls gets **-1/-1 until end of
  turn**." vs def mode 1 `Effect::AddCounter { target: DeclaredTarget { index: 0 }, counter:
  CounterType::MinusOneMinusOne, count: 1 }` at `shambling_ghast.rs:49-53`.
  Effect on play: a **permanent** -1/-1 counter instead of a one-turn P/T change. A creature that would
  have survived past end of turn stays shrunk forever; the counter is a real counter, so it is visible to
  proliferate, to "remove a counter" costs, and it annihilates with +1/+1 counters under CR 704.5q —
  none of which a duration-based P/T modification does.
- DEFECT 3 (stored oracle text does not match the printed card, three ways): the field at
  `shambling_ghast.rs:17-20` (a) states a **Decayed** reminder clause the card does not have, (b) says
  "When Shambling Ghast **enters**" while the printed trigger — and the def's own
  `trigger_condition: TriggerCondition::WhenDies` at line 30 — is **dies**, and (c) drops both "**an
  opponent controls**" and "**until end of turn**" from the -1/-1 mode.
  Effect on play: none directly, but it is the `jadar_ghoulcaller_of_nephalia` failure mode — a stored
  text that was never right, from which any future blocker note or triage reasons incorrectly. Note that
  the def's *behaviour* is right where the stored text is wrong (it really is `WhenDies`, and the target
  filter really is `Opponent`), so the field is a trap for readers, not a description of the code.
- DEFECT 4 (found here, not in the spot-check — file header comment): `shambling_ghast.rs:1` reads
  "// Shambling Ghast — {B}, Creature — Zombie 1/1; **Decayed**." — the phantom keyword is asserted a
  second time in prose, so a reader who checks the comment against the code finds them consistent and
  concludes both are right.
- WATCH: as with Retreat to Kazandu, the mode-1 target requirement sits at the ability level with
  `mode_targets: None` (`shambling_ghast.rs:33-39`), so the death trigger appears to require an
  opponent's creature even when mode 0 (Treasure) would be chosen. Same `mode_targets` seed.

---

### Smuggler's Copter — **CLASS D** (independently re-verified; I agree with the allegation)
- BASELINE row: `discard_cards`
- declared completeness: `Complete` (explicit, `smugglers_copter.rs:83`)
- MCP printed: `{2}`, `Artifact — Vehicle`, 3/3, keywords `["Flying","Crew"]`
- MCP printed oracle text: "Flying\nWhenever this Vehicle attacks or blocks, **you may draw a card. If you do, discard a card.**\nCrew 1 (Tap any number of creatures you control with total power 1 or more: This Vehicle becomes an artifact creature until end of turn.)"
- stored `oracle_text` field: **DIFFERS (ordering + templating)** — the def lists Flying, then Crew 1
  with reminder text, then "Whenever **Smuggler's Copter** attacks or blocks, you may draw a card. If you
  do, discard a card." The printed order is Flying / attack-or-block trigger / Crew 1, and the printed
  trigger says "this Vehicle", not the card name. Note the stored text **does** contain the printed
  "you may", which the code does not — the field and the implementation disagree with each other.
- verdict rationale: `{2}`, `Artifact — Vehicle`, 3/3, `Flying`, `Crew(1)` and the two trigger conditions
  (`WhenAttacks`, `WhenBlocks`) are all correct. The defect is that the printed optionality is gone: the
  loot is authored as an unconditional two-step sequence.
- DEFECT 1: printed "Whenever this Vehicle attacks or blocks, **you may draw a card. If you do, discard a
  card.**" vs def `effect: Effect::Sequence(vec![Effect::DrawCards { player: Controller, count: Fixed(1) },
  Effect::DiscardCards { player: Controller, count: Fixed(1) }])` — twice, at
  `crates/card-defs/src/defs/smugglers_copter.rs:35-44` (`WhenAttacks`) and `:54-63` (`WhenBlocks`).
  There is no `MayPayThenEffect`, no `Conditional`, no mode: the "may" is simply absent.
  Effect on play: the controller is **forced to loot on every attack and every block**. Three concrete
  wrong outcomes, in increasing severity: (i) a player who wants to keep a full hand must discard a card
  they wanted anyway; (ii) with exactly one card in hand it can be forced away; (iii) **with an empty
  library the forced draw loses the game** under CR 104.3c — the printed card lets you simply decline.
  In a Commander game where a Copter attacks every turn for many turns, this is a systematic, repeated
  divergence, not an edge case.
- Note: this is also the exact case the `decision_gate.rs` module doc (lines 21-25) already names as the
  class the gate is structurally blind to (OOS-DP10-9) — the def hits `discard_cards` only *incidentally*,
  via the second half of the sequence; the dropped "may" itself leaves no trace in any decision row.
- WATCH: the printed card has **one** triggered ability with two trigger events; the def splits it into
  two abilities. Mechanically equivalent in all reachable board states (a creature cannot both attack and
  block in the same combat) and not scored as a defect — noted only so a fixer does not "consolidate" and
  accidentally drop the block half.

---

### Spell Pierce — **CLASS B**
- BASELINE row: `counter_unless_pays`
- declared completeness: `Complete` (BY `#[default]` — no field declared)
- MCP printed: `{U}`, `Instant`, keywords `[]`
- MCP printed oracle text: "Counter target noncreature spell unless its controller pays {2}."
- stored `oracle_text` field: **matches** (verbatim).
- verdict rationale: mana cost `{U}` (blue: 1, no generic) correct. `TargetRequirement::TargetSpellWithFilter`
  with `non_creature: true` is "target **noncreature** spell" (KI-1-adjacent filter present and correct,
  not an unfiltered `TargetSpell`). `Effect::CounterUnlessPays` with `Cost::Mana(generic: 2)` is
  "unless its controller pays **{2}**", and CR 118.12a routes the payment decision to the **spell's
  controller** — the correct player, not the caster. The only auto-choice is whether that controller pays;
  the documented default (decline → countered) is a legal answer, i.e. the class-B premise.
- WATCH: none.

---

### Springbloom Druid — **CLASS B**
- BASELINE row: `may_pay_then_effect`
- declared completeness: `Complete` (BY `#[default]` — no field declared)
- MCP printed: `{2}{G}`, `Creature — Elf Druid`, 1/1, keywords `[]`
- MCP printed oracle text: "When this creature enters, you may sacrifice a land. If you do, search your library for up to two basic land cards, put them onto the battlefield tapped, then shuffle."
- stored `oracle_text` field: **matches** (verbatim, modern templating).
- verdict rationale: this is the *correct* rendering of the pattern Smuggler's Copter gets wrong — the
  printed "**you may** sacrifice a land. **If you do**, ..." is authored as
  `Effect::MayPayThenEffect { cost: Cost::Sacrifice(land filter), payer: Controller, then: ... }`, so the
  optionality is preserved in the DSL and the whole search is correctly gated on the sacrifice actually
  happening. Mana cost, type line and P/T all match; lands enter **tapped** as printed; a trailing
  `Effect::Shuffle` supplies "then shuffle". The auto-choices (whether to pay, which land, which basics)
  are the BASELINE row.
- WATCH: same "up to two" → two sequential single-card `SearchLibrary` approximation as Roiling Regrowth
  (`springbloom_druid.rs:38-53`), for the same OOS-DP9-3 reason. Not scored as D.

---

### Staff of Compleation — **CLASS D** (narrow — see the downgrade note)
- BASELINE row: `proliferate`
- declared completeness: `Complete` (BY `#[default]` — no field declared; and the def **says so about
  itself** at `staff_of_compleation.rs:95`: "// PB-EF12 (EF-W-PB2-3): un-marked, see birds_of_paradise.rs
  for the fix." The marker was flagged as missing eight-plus batches ago and never added.)
- MCP printed: `{3}`, `Artifact`, keywords `["Proliferate"]`
- MCP printed oracle text: "{T}, Pay 1 life: Destroy target permanent **you own**.\n{T}, Pay 2 life: Add one mana of any color.\n{T}, Pay 3 life: Proliferate.\n{T}, Pay 4 life: Draw a card.\n{5}: Untap this artifact."
- stored `oracle_text` field: **matches** (verbatim, including "you own").
- verdict rationale: all five abilities are present with the correct costs (`Tap` + `PayLife(1/2/3/4)`,
  and `{5}` with no tap for the untap), the correct effects, and no timing restrictions the printed card
  does not have. Mana cost `{3}` and type `Artifact` are right. The proliferate auto-choice is the
  recorded class-B deviation. The single deviation is the first ability's target filter: **"you own"
  authored as "you control"**.
- DEFECT 1: printed "{T}, Pay 1 life: Destroy target permanent **you own**." vs def
  `targets: vec![TargetRequirement::TargetPermanentWithFilter(TargetFilter { controller:
  TargetController::You, .. })]` at `crates/card-defs/src/defs/staff_of_compleation.rs:30-33`.
  Effect on play: ownership and control diverge under any control-change effect, which in a 4-player
  Commander game is routine (Mind Control, Agent of Treachery, Act of Treason, Blatant Thievery, threaten
  effects, Commander-deck staples). Two symmetric errors: (i) a permanent **you own but an opponent
  controls** — precisely the case the printed card is *designed* for, letting you destroy your own stolen
  permanent rather than let the thief keep it — is **not** a legal target for the def; and (ii) a
  permanent **you control but do not own** (something you stole) **is** a legal target for the def, and
  destroying it sends it to its owner's graveyard, an outcome the printed card never permits.
- DSL note: `TargetFilter` has **no owner field** — `TargetController` is `{ Any, You, Opponent,
  DamagedPlayer }` (`card_definition.rs:3242-3254`) and the only `owner:` in the file is on `ZoneTarget`.
  So "permanent you own" is **not expressible today**. The correct disposition is therefore probably an
  explicit `Completeness::Partial` marker with a blocker note (which would also discharge the eight-batch-old
  "un-marked" comment), not a def rewrite.
- Downgrade note (offered deliberately, per the be-conservative instruction): this is the mildest D in
  the batch. In the majority of board states you own everything you control and the two filters coincide,
  and the deviation is an unexpressible-in-DSL gap rather than a mis-authoring. If the triage's D bar is
  "wrong in ordinary play without an enabling effect", this one falls below it and should be recorded as
  a WATCH plus a marker seed. I score it D because "wrong filter" is explicitly in the class-D list, the
  divergence is *bidirectional* (both a false negative and a false positive on targeting), and a
  `Complete`-by-`#[default]` def is deck-legal today.

---

### Stubborn Denial — **CLASS B**
- BASELINE row: `counter_unless_pays`
- declared completeness: `Complete` (BY `#[default]` — no field declared)
- MCP printed: `{U}`, `Instant`, keywords `["Ferocious"]`
- MCP printed oracle text: "Counter target noncreature spell unless its controller pays {1}.\nFerocious — If you control a creature with power 4 or greater, counter that spell instead."
- stored `oracle_text` field: **matches** (verbatim, including the em-dash Ferocious label).
- verdict rationale: mana cost `{U}`, type `Instant`, and the `non_creature: true` spell-target filter all
  correct. The ferocious clause is authored as `Effect::Conditional { condition:
  Condition::YouControlPermanent(TargetFilter { has_card_type: Creature, min_power: Some(4), .. }) }`
  wrapping an unconditional `CounterSpell` (true branch) and `CounterUnlessPays { cost: {1} }` (false
  branch) — which is the correct structure and, importantly, the correct **timing**: ferocious is checked
  on resolution (the printed clause is part of the spell's effect, not an intervening-if), and
  `Effect::Conditional` evaluates at resolution. `min_power: Some(4)` is "power **4 or greater**"
  (inclusive), and both branches target the same `DeclaredTarget { index: 0 }` — "counter **that spell**
  instead". Same class-B auto-choice as Spell Pierce (whether the spell's controller pays).
- WATCH: none.

---

## Batch notes

**Templating drift policy.** Four defs (Raffine's Informant, Risen Reef, Roalesk, Smuggler's Copter)
store an `oracle_text` that uses pre-2024 templating — the card's own name where the current printing
says "this creature"/"this Vehicle", and/or "enters the battlefield" where the current printing says
"enters". I did **not** score these as class D: the wording is semantically identical, it reflects a real
earlier printed wording, and it appears to be corpus-wide. It is distinct from Shambling Ghast's and
Radstorm's stored-text defects, which assert clauses the card has **never** had (a Decayed reminder, an
"enters" trigger on a dies-triggered ability, an obsolete Storm reminder sentence) — those are the
`jadar_ghoulcaller_of_nephalia` class and are scored. A one-time corpus-wide templating sweep would be a
separate, cheap, cosmetic exercise.

**`#[default]` marker exposure in this batch: 10 of 14.** Radstorm, Raiders' Wake, Retreat to Kazandu,
Risen Reef, Roiling Regrowth, Satyr Wayfinder, Spell Pierce, Springbloom Druid, Staff of Compleation and
Stubborn Denial declare **no `completeness` field at all** and are `Complete` — and therefore deck-legal —
purely by the derive. Two of the batch's five class-D defs (Radstorm, Satyr Wayfinder) are in that group,
and Staff of Compleation carries an in-file comment acknowledging it is un-marked. This is direct
supporting evidence for the standing "`#[default] Completeness::Complete` is a silent-defect generator"
finding, and for the cheap corpus-wide question CLAUDE.md flags as never having been asked. (The four
explicit-marker defs are Raffine's Informant, Roalesk, Shambling Ghast, Smuggler's Copter — note that two
of *those* are class D too, so an explicit marker is no protection; the point is that the un-marked group
has never been looked at at all.)

**`mode_targets` seed (2 defs).** Retreat to Kazandu and Shambling Ghast both attach a mode-specific
target requirement at the ability level with `mode_targets: None`, so a mode that needs no target may
still be gated on one existing. Recorded as WATCH on both rather than D, because it is one shared
machinery gap rather than two card errors.

SUMMARY batch6: 9 class-B, 5 class-D (Radstorm, Satyr Wayfinder, Shambling Ghast, Smuggler's Copter, Staff of Compleation), 6 watch
