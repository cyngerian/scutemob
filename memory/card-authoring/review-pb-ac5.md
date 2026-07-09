# Card Review: PB-AC5 — alt-costs, timing keywords, exile_instead add-on

**Reviewed**: 2026-07-08
**Reviewer**: card-batch-reviewer (Opus)
**Scope**: 6 primary (new), 21 secondary (exile_instead touch), 3 blocked-marker verifications
**Findings**: 0 HIGH, 1 MEDIUM, 3 LOW

All oracle text, mana costs, type lines, P/T, and color identities in the primary
batch match Scryfall exactly. The `exile_instead` add-on regressed nothing. All three
deliberately-blocked markers are honest about the correct remaining gap.

---

## PRIMARY BATCH (newly authored)

### Card 1: Timeline Culler — CLEAN
- Oracle/mana/type/P/T match: YES ({B}{B}, Creature — Drix Warlock, 2/2)
- Warp cost authored as composite: `cost: {B}` + `costs: vec![Cost::PayLife(2)]` — matches
  "Warp—{B}, Pay 2 life." YES
- `from_graveyard: true` correctly encodes "you may cast this card from your graveyard
  using its warp ability." YES
- Dual-def present: `Keyword(Warp)` marker + `AltCastAbility{ kind: Warp, .. }`. YES
- No findings.

### Card 2: Dimir Infiltrator — CLEAN
- Oracle/mana/type/P/T match: YES ({U}{B}, Spirit, 1/3)
- Transmute dual-def present: `Keyword(Transmute)` + activated ability. YES (KI-6 satisfied)
- Cost `Sequence([Mana {1}{U}{B}, DiscardSelf])`, `SorcerySpeed` timing — matches
  "{1}{U}{B}, Discard this card ... Transmute only as a sorcery." YES
- Search filter `min_cmc: Some(2), max_cmc: Some(2)` — Dimir Infiltrator's own MV is 2,
  so hardcoding is faithful for this card; comment correctly scopes the general
  "same-MV-as-source dynamic" filter as out of PB-AC5 scope. Acceptable. YES
- No findings.

### Card 3: Combat Celebrant — CLEAN
- Oracle/mana/type/P/T match: YES ({2}{R}, Human Warrior, 4/1)
- `Keyword(Exert)` (optional attack cost) + `Triggered{ WhenExertedAsAttacks }` linked
  trigger — matches "you may exert it as it attacks. When you do, ..." (CR 701.43d/607.2h).
  Trigger fires only on the exert choice, not every attack. YES
- Effect: `ForEach(EachOtherCreatureYouControl -> Untap)` + `AdditionalCombatPhase` —
  matches "untap all other creatures you control and after this phase, there is an
  additional combat phase." Excludes self correctly. YES
- No findings.

### Card 4: Force of Will — CLEAN
- Oracle/mana/type match: YES ({3}{U}{U}, Instant)
- Pitch cost `costs: vec![PayLife(1), ExileFromHand{Blue}]`, `opponents_turn_only: false`
  — FoW has NO "not your turn" restriction; correctly distinguished from the other Forces. YES
- CounterSpell `exile_instead: false` — "Counter target spell." (plain bin). YES
- No findings.

### Card 5: Force of Vigor — CLEAN (1 LOW)
- Oracle/mana/type match: YES ({2}{G}{G}, Instant)
- Pitch `costs: vec![ExileFromHand{Green}]`, `opponents_turn_only: true` — matches
  "If it's not your turn, you may exile a green card from your hand ...". YES
- Target: `UpToN{ count: 2, inner: TargetPermanentWithFilter{ has_card_types:
  [Artifact, Enchantment] } }`. Confirmed `has_card_types` is OR-semantics
  (card_definition.rs:2713 "must have at least one of these types"), so this correctly
  reads "artifact and/or enchantment." YES
- **F1 (LOW)**: force_of_vigor.rs:28-37 — effect is a fixed `Sequence` of two
  `DestroyPermanent` at `DeclaredTarget{index:0}` and `{index:1}`. When the caster
  chooses 0 or 1 targets under UpToN, index 1 (or both) references no declared target.
  This is the established codebase pattern for "up to two" and UpToN is documented
  complete (PB-T / PB-AC4), so missing indices should no-op — but worth a one-line
  confirmation that a resolved effect over an absent DeclaredTarget silently skips
  rather than panics. Not a correctness defect if the engine handles it as elsewhere.

### Card 6: Force of Negation — CLEAN
- Oracle/mana/type match: YES ({1}{U}{U}, Instant)
- Pitch `costs: vec![ExileFromHand{Blue}]`, `opponents_turn_only: true`. YES
- Target filter `TargetSpellWithFilter{ non_creature: true }` — "target noncreature
  spell." YES
- CounterSpell `exile_instead: true` — matches "exile it instead of putting it into
  its owner's graveyard." YES. Note: oracle has no "cast during your turn" clause
  (the task prompt's hypothetical "if it was cast during your turn" wording does NOT
  appear in current oracle); the def correctly omits any such condition.
- No findings.

---

## SECONDARY BATCH — exile_instead add-on (21 defs)

All 21 verified. Every def has `exile_instead: false` EXCEPT force_of_negation
(`true`). No counter def whose oracle sends the spell to the graveyard was given
`true`. No regression introduced.

Files confirmed false: abjure, arcane_denial, archmages_charm, counterspell,
cryptic_command, dispel, dovins_veto, fierce_guardianship, flare_of_denial,
force_of_will, mana_drain, memory_lapse, mental_misstep, negate, pyroblast,
red_elemental_blast, rewind, saw_it_coming, stubborn_denial, swan_song.
File confirmed true: force_of_negation (matches oracle).

Special-destination checks (behaviors the bool cannot express):

- **F2 (LOW)** — memory_lapse.rs:14-18: oracle puts the countered spell "on top of its
  owner's library instead of into that player's graveyard." `exile_instead: false` is
  the correct value (it should NOT exile), but the bool cannot express top-of-library,
  so the card still bins the spell (wrong destination). This is a PRE-EXISTING partial
  implementation, already documented with a TODO ("requires counter-to-top variant"),
  NOT a PB-AC5 regression. Flagging per the "bool can't express it" instruction. The
  counter-to-top primitive remains a genuine open gap.

- mana_drain.rs: current authoritative oracle sends the spell to the graveyard (the
  delayed {C} rider is a separate documented TODO). `exile_instead: false` is correct;
  the prompt's "exiles it in some printings" does not match current oracle. No finding.

- arcane_denial / flare_of_denial / swan_song / dovins_veto: all counter-to-graveyard
  in current oracle; their extra clauses (draws, sacrifice alt cost, token, can't-be-
  countered) are orthogonal to the destination bool. `false` correct. No findings.

---

## BLOCKED-MARKER VERIFICATION

### Starfield Shepherd — marker HONEST (1 MEDIUM)
- Oracle/mana/type/P/T match: YES ({3}{W}{W}, Angel, 3/2)
- Marker correctly names the remaining gap: the ETB "basic Plains card OR creature card
  with mana value 1 or less" needs cross-group OR + max-mana-value TargetFilter support,
  which does not exist. It correctly states Warp is NOT the blocker (Warp shipped in
  PB-AC5). The specific check ("blocked only by the ETB disjunctive search, not by Warp")
  PASSES.
- **F3 (MEDIUM)**: The def authors `Keyword(Flying)` but omits the now-expressible Warp
  ability (`Keyword(Warp)` + `AltCastAbility{ Warp {1}{W} }`), even though the file
  header says "Warp primitive shipped in PB-AC5." Compare Arena of Glory below, which
  DOES author its independent, now-expressible abilities and blocks only the true gap.
  Starfield is inconsistent: it authors one keyword and drops another expressible one.
  Either (a) author Warp too (the task's stated expectation — "the rest of the card is
  authored, not stubbed"), or (b) if the card is left fully blocked because its MANDATORY
  ETB trigger would silently never fire (invariant #9 corrupted-history risk), then it
  should not be authored/registered as partially playable at all. Current state is a
  half-authored middle ground. Not a correctness defect in what IS present (Flying is
  right), but the "rest of the card is authored" expectation is not met.

### Force of Despair — marker HONEST, CLEAN
- Oracle/mana/type match: YES ({1}{B}{B}, Instant)
- `abilities: vec![]` with ENGINE-BLOCKED comment. Marker correctly names the gap:
  "Destroy all creatures that entered this turn" needs an entered-this-turn DestroyAll
  predicate that does not exist; the pitch alt cost IS implemented but is deliberately
  NOT attached alone (W6 no-partial-authoring). Correct and consistent. Marker states
  the pitch cost is not the blocker — PASSES.
- No findings.

### Arena of Glory — marker HONEST, CLEAN
- Oracle/mana/type match: YES (Land, no mana cost)
- ETB-tapped: `Replacement{ EntersTapped, unless: ControlLandWithSubtypes([Mountain]) }`
  — matches "enters tapped unless you control a Mountain." YES (KI-13 satisfied; oracle
  DOES require the ETB-tapped condition, and it is present with the correct unless).
- `{T}: Add {R}`: `AddMana mana_pool(0,0,0,1,0,0)` — WUBRGC order, red slot = 1. Correct. YES
- Third ability (exert for {R}{R} + mana-spend-conditional haste) blocked. Marker
  correctly names the gap: mana-spend-provenance -> delayed-effect on the paid-for object,
  which ManaRestriction cannot express. It correctly states Exert (Cost::Exert) is NOT the
  blocker (implemented in PB-AC5). PASSES. Omitting one optional activated ability does
  not corrupt the two authored abilities. Acceptable partial (parallel to how Arena
  authors its safe abilities while Force of Despair, whose only ability is blocked,
  authors nothing).
- No findings.

---

## Summary
- **HIGH**: none.
- **MEDIUM (1)**: F3 — Starfield Shepherd omits the now-expressible Warp ability while
  authoring Flying; "rest of card authored" expectation not met (marker itself is honest).
- **LOW (2)**: F1 — Force of Vigor fixed two-index Sequence under UpToN (confirm absent-
  target no-op); F2 — Memory Lapse still bins instead of top-of-library (pre-existing,
  documented TODO; bool cannot express it — not a PB-AC5 regression).
- **Clean cards**: Timeline Culler, Dimir Infiltrator, Combat Celebrant, Force of Will,
  Force of Negation, Force of Despair, Arena of Glory; all 21 exile_instead touches.
- **Regressions from exile_instead add-on**: none.
