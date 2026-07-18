# Card Review: W-PB2 Batch 4 — 12 triggered-ability defs

**Reviewed**: 2026-07-17
**Cards**: 12 (10 Complete, 1 partial, 1 inert)
**Findings**: 0 HIGH, 2 MEDIUM, 6 LOW
**Verdict**: 12 PASS, 0 FIX, 0 DEMOTE

DSL gaps cited by scourge/dragon_tempest were verified against source, not assumed:
- `Effect::DealDamage` (`card-types/src/cards/card_definition.rs:1330`) has exactly two
  fields — `target`, `amount`. No `source:` override. Source-override gap is REAL.
- `EffectFilter` (`card-types/src/state/continuous_effect.rs:67-246`) has no
  `TriggeringCreature` variant (closest are `DeclaredTarget`, `Source`). Haste-grant gap is REAL.
- ForEach-EachCreature damage pattern in spiteful_banditry matches the established
  `chandra_flamecaller.rs:61-65` reference (DealDamage → DeclaredTarget{index:0}, XValue).
- `GameRestriction::OpponentsCantCastDuringYourTurn` exists (`stubs.rs:565`).

---

## Card 1: Kalastria Highborn — PASS
- Oracle match: YES (semantic); Types: YES ({B}{B} Vampire Shaman 2/2); DSL: YES
- `WheneverCreatureDies{ controller:Some(You), exclude_self:false, filter:Vampire }` correct —
  inclusive "this or another Vampire," Kalastria matches her own death (def line 26-34).
- `Effect::MayPayThenEffect{ cost:{B}, then: LoseLife(target,2) + GainLife(you,2) }` — correct
  optional-pay shape (NOT the `Choose`/`MayPayOrElse` stub family). Targets `[TargetPlayer]`
  (line 53), LoseLife → `DeclaredTarget{index:0}` (line 43). All correct.
- **F1 (LOW)**: oracle_text field writes "Kalastria Highborn" where current oracle templates
  "this creature" (line 15). Cosmetic; behavior unaffected.

## Card 2: Nether Traitor — PASS
- Oracle match: YES; Types: YES ({B}{B} Spirit 1/1); DSL: YES
- Haste + Shadow keyword markers present (lines 23-24). `trigger_zone:Some(Graveyard)` IS set
  (line 52) — required or the ability can't fire from the graveyard. `exclude_self:true` (line 32)
  correct for "another creature." `MayPayThenEffect{ {B}, MoveZone(Source→Battlefield untapped) }`
  correct (lines 36-47).
- **F2 (MEDIUM, KI-11, recommend PASS + document, NOT demote)**: oracle says "put into **your
  graveyard** from the battlefield" — a card always goes to its **owner's** graveyard (CR 404.3),
  so this clause keys on **ownership**. The def uses `controller:Some(You)`, i.e. control. These
  diverge only for stolen creatures: (a) a creature you OWN but an opponent controls dies → to
  your graveyard → should trigger, but `controller:You` misses it; (b) a creature you STOLE dies
  → to opponent's graveyard → should NOT trigger, but `controller:You` fires it. The DSL has no
  owner-based death trigger, and control==own in the overwhelming majority of Commander boards, so
  this is an acceptable approximation — but it is undocumented in the def. Recommend adding a note
  rather than demoting. (boggart below is NOT affected — its oracle says "you control," so
  control-keying is exactly right there.)

## Card 3: Boggart Shenanigans — PASS
- Oracle match: YES; Types: YES ({2}{R} Kindred Enchantment — Goblin); DSL: YES
- Trigger `WheneverCreatureDies{ controller:Some(You), exclude_self:true, filter:Goblin }`
  (lines 34-42) — correct ("another Goblin you control"). Target
  `TargetPlayerOrPlaneswalker` (line 48), DealDamage 1 → `DeclaredTarget{index:0}`. Correct.
- **Adjudication (may→mandatory)**: ACCEPTABLE. Oracle "you MAY have this deal 1 damage" is a
  non-gated optionality with no cost, which the DSL can't express (only `MayPayThenEffect` gates
  on a real cost). Modeled mandatory per the avenger/strictly-beneficial-ping convention: an
  opponent player is always a legal target (no hexproof on players), and 1 damage to an opponent
  has no downside. Author's own LOW caveat (can't decline to avoid revealing a target) is the only
  residual, and it's genuinely LOW. Does not warrant `partial`.

## Card 4: Spiteful Banditry — PASS
- Oracle match: YES; Types: YES ({X}{R}{R} Enchantment, x_count:1 red:2); DSL: YES
- **MCP confirms NO "may"** and exactly TWO clauses (ETB damage + Treasure-on-opponent-death);
  no sacrifice-Treasures clause exists on the card. Mandatory modeling is correct.
- ETB: `ForEach{EachCreature} → DealDamage{DeclaredTarget{0}, XValue}` (lines 25-32) — matches
  the certified `chandra_flamecaller.rs` reference pattern exactly.
- Treasure trigger: `once_per_turn:true` (line 44) + `WheneverCreatureDies{controller:Opponent}`
  (line 46) + `CreateToken{treasure_token_spec(1)}` — all set correctly. Count present on the
  spec (KI-16 clean).

## Card 5: Dwynen, Gilt-Leaf Daen — PASS
- Oracle match: YES; Types: YES (Legendary supertype present, line 17; Elf Warrior 3/4); DSL: YES
- Reach keyword. Static +1/+1 via `OtherCreaturesYouControlWithSubtype(Elf)` (line 33) — "Other."
- Attack trigger: `GainLife{ AttackingCreatureCount{ controller:Controller, filter:Elf } }`
  (lines 44-53). Decoy check: a non-Elf attacker is excluded by `has_subtype:Elf`. Dwynen herself
  (an attacking Elf) is correctly counted (no exclude on the count). Correct.

## Card 6: Elderfang Venom — PASS
- Oracle match: YES; Types: YES ({2}{B}{G} Enchantment); DSL: YES
- Deathtouch static via `AttackingCreaturesYouControlWithSubtype(Elf)` (line 26) — correct
  filter for "Attacking Elves you control have deathtouch."
- Death trigger `WheneverCreatureDies{You, Elf}` → `Sequence[ ForEach{EachOpponent} LoseLife 1,
  GainLife(you,1) ]` (lines 45-56). Matches "each opponent loses 1 life and you gain 1 life."

## Card 7: Wolverine Riders — PASS
- Oracle match: YES; Types: YES ({4}{G}{G} Elf Warrior 4/4); DSL: YES
- Upkeep token `AtBeginningOfEachUpkeep` → 1/1 green Elf Warrior (lines 23-52). Token spec colors,
  subtypes, P/T all correct.
- ETB trigger `WheneverCreatureEntersBattlefield{ filter:{Elf, controller:You}, exclude_self:true }`
  → `GainLife{ ToughnessOf(TriggeringCreature) }` (lines 54-67). Correct — "another Elf ... its
  toughness."

## Card 8: Roalesk, Apex Hybrid — PASS
- Oracle match: mostly (see F3); Types: YES (Legendary Human Mutant 4/5); DSL: YES
- Flying + Trample. ETB `AddCounter{ 2x +1/+1, DeclaredTarget{0} }` with target
  `TargetCreatureWithFilter{ controller:You, exclude_self:true }` (lines 43-47) — "another target
  creature you control" correctly excludes Roalesk. Required target (not "up to"); correct.
- Dies: `Sequence[ Proliferate, Proliferate ]` (line 56) — "proliferate, then proliferate again."
- **F3 (LOW)**: oracle_text expands self-name to "Roalesk, Apex Hybrid" (oracle prints "Roalesk")
  and drops the proliferate reminder text (lines 22-24). Cosmetic.

## Card 9: Brokers Ascendancy — PASS
- Oracle match: YES; Types: YES ({G}{W}{U} Enchantment); DSL: YES
- **MCP confirms a SINGLE clause** — the end-step counter distributor — with **no hexproof / no
  static-buff clause** on this printing. No demotion warranted.
- `AtBeginningOfYourEndStep` → `Sequence[ ForEach{EachCreatureYouControl} +1/+1,
  ForEach{EachPermanentMatching(Planeswalker,You)} +loyalty ]` (lines 23-47). Correct split of the
  "+1/+1 on each creature ... loyalty on each planeswalker."

## Card 10: Voice of Victory — PASS
- Oracle match: YES; Types: YES ({1}{W} Human Bard 1/3); DSL: YES
- Mobilize 2 modeled as `WhenAttacks → CreateToken{ 2x 1/1 red Warrior, tapped:true,
  enters_attacking:true, sacrifice_at_end_step:true }` (lines 25-53) — faithful to the reminder
  text (two tapped+attacking tokens, sac at next end step). Self-contained; no double-count risk.
- Static `StaticRestriction{ OpponentsCantCastDuringYourTurn }` (lines 55-57) — variant verified
  in `stubs.rs:565`.
- **F4 (LOW)**: no `KeywordAbility::Mobilize` marker accompanies the hand-rolled trigger. Behavior
  is fully correct without it; only relevant if another card ever queries "has Mobilize." Not a
  KI-6 dual-def violation because the behavior is inlined, not deferred to a keyword handler.

## Card 11: Scourge of Valkas — PASS (partial marker honest)
- Oracle match: YES; Types: YES ({2}{R}{R}{R} Dragon 4/4). DSL of the authored half: correct.
- Self-ETB half: `WhenEntersBattlefield → DealDamage{ TargetAny,
  PermanentCount{Creature+Dragon, controller:You} }` (lines 31-49). Correct: when Scourge enters
  it is on the battlefield and counts itself; `ctx.source == Scourge == "it"`, so the implicit
  DealDamage source matches oracle. `{R}: +1/+0` activated ability via `Source` filter (lines
  53-72) correct.
- **Partial marker is HONEST (F5, informational)**: the residual "another Dragon you control
  enters → **it** deals X" half needs the ENTERING Dragon to be the damage source, and
  `Effect::DealDamage` has no source-override (verified — source always = ctx.source = Scourge).
  Shipping it would misattribute the source (protection/redirection/"a source you control"). The
  note (lines 74-86) accurately describes both the authored half and the real gap. No stale-gap
  (KI-3) issue.

## Card 12: Dragon Tempest — PASS (inert taxonomy correct)
- Oracle match: YES; Types: YES ({1}{R} Enchantment). `abilities: vec![]`, no other
  behavior-bearing field → registers no behavior → **inert is the correct taxonomy** (not partial).
- **Both gaps in the note are REAL (verified)**: (1) "it gains haste" needs
  `EffectFilter::TriggeringCreature` on `ContinuousEffectDef.filter` — no such variant exists;
  (2) "it deals X damage" needs entering-Dragon as source — `Effect::DealDamage` has no source
  override, and unlike Scourge there is NO self-ETB case (Dragon Tempest isn't a Dragon), so every
  firing would misattribute. Note (lines 38-46) names both gaps and the ogre_battledriver /
  shared_animosity precedent. No stale-gap (KI-3) issue.

---

## Summary
- **Cards with issues (all documentation-only, none blocking Complete)**: nether_traitor (F2
  MEDIUM controller-vs-owner on "your graveyard" — recommend a doc note, common case correct),
  kalastria/roalesk (LOW oracle self-name/reminder cosmetics), voice_of_victory (LOW missing
  Mobilize marker), boggart (may→mandatory, adjudicated acceptable).
- **Clean cards**: spiteful_banditry, dwynen, elderfang_venom, wolverine_riders,
  brokers_ascendancy.
- **partial/inert markers**: scourge_of_valkas (partial) and dragon_tempest (inert) are both
  HONEST — both cited DSL gaps confirmed against source. No demotions, no stale-gap TODOs.
- **Gated-stub scan (Choose / MayPayOrElse / AddManaChoice / AddManaAnyColor in a Complete def)**:
  NONE found. kalastria's `MayPayThenEffect` is the real optional-pay primitive, not a stub.
- **No HIGH findings. No FIX or DEMOTE actions required.**
