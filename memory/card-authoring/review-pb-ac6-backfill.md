# PB-AC6 Card Backfill Review

**Reviewed**: 2026-07-09
**Scope**: `git diff 83c6c7d3..HEAD -- crates/engine/src/cards/defs/` (PB-AC6 backfill)
**Method**: every card's oracle text pulled from `lookup_card` (authoritative); DSL claims
verified against engine source (`EffectTarget`/`PlayerTarget` enums, `Effect::AddCounter`
execution, modal-triggered-ability resolution, first-main/postcombat trigger dispatch).

**Findings**: 2 HIGH, 1 MEDIUM, 5 LOW

---

## HIGH

### H1 — Tectonic Giant: dud `Effect::Nothing` mode + false "modal-on-trigger unsupported" marker
`crates/engine/src/cards/defs/tectonic_giant.rs:18-54`

The header comment and the PB-AC6-added parenthetical both assert modal triggered
abilities are inexpressible ("ModeSelection is only wired for spells, not triggered
abilities … [the becomes-target half] stays unauthored only because the modal effect
below cannot be represented"). **This is false on two counts:**

1. The card *does* author a modal triggered ability (`modes: Some(ModeSelection{…})`).
2. The engine *does* resolve modal triggered abilities — `rules/resolution.rs:2047-2086`
   (CR 700.2b path) substitutes the chosen mode effect at resolution, with a bot
   fallback of mode 0. Verified Mode 0 works: `ForEach{over: EachOpponent}` injects each
   opponent as `DeclaredTarget{index:0}` into the inner context
   (`effects/mod.rs:2919-2933`), so "deals 3 damage to each opponent" resolves correctly.

The actual problem: **Mode 1 is `Effect::Nothing`** (line 47), a no-op placeholder for the
inexpressible impulse-play clause ("exile top two, play one until end of your next turn").
A player who chooses Mode 1 pays the trigger and gets nothing — wrong game state. This is
exactly the "near-miss / no-op standing in for an inexpressible clause" the W6 policy
forbids (KI-10, and the task's "no approximations = HIGH").

Oracle: *"Whenever this creature attacks or becomes the target of a spell an opponent
controls, choose one — • This creature deals 3 damage to each opponent. • Exile the top
two cards of your library. Choose one of them. Until the end of your next turn, you may
play that card."*

Fix: since one of the two modes is inexpressible, a "choose one" cannot be authored
without a dead mode — set `abilities: vec![]` with an ENGINE-BLOCKED marker naming the
true gap (impulse-play effect), exactly as `black_market_connections.rs` does for the same
class of card. Do **not** leave the modal with a `Nothing` branch. Separately, correct the
PB-AC6 marker: modal-on-trigger and `WhenBecomesTarget` are both supported; the sole
blocker is the impulse-play effect (and modal-target routing for the becomes-target half).

Note on provenance: the modal body appears to predate PB-AC6 (PB-AC6 added only the
parenthetical). It is nonetheless a live wrong-state defect and the PB-AC6 marker actively
misdescribes it, so it is in scope.

### H2 — Mindbreak Trap: "any number of target spells" approximated as a single target
`crates/engine/src/cards/defs/mindbreak_trap.rs:21-28`

Oracle: *"Exile any number of target spells."* The def authors
`ExileObject { target: DeclaredTarget{index:0} }` with `targets:
vec![TargetRequirement::TargetSpell]` — a **single** target spell. The card's own comment
acknowledges "variable target counts are not supported," then authors around the gap
anyway. Result: the card can exile exactly one spell instead of a whole chain — a
materially different (strictly weaker) game action. This is a near-miss standing in for an
inexpressible clause (HIGH per task directive; W6 "no approximations").

Fix: `abilities: vec![]` with an ENGINE-BLOCKED marker until (a) variable target counts
and (b) the Trap alt-cost (`AltCostKind::Trap`) exist. The PB-AC6 marker is otherwise
correct that `Condition::OpponentCastNSpells(3)` now exists and the missing piece is the
alt-cost wrapper, not the count.

Provenance: the single-target Spell body likely predates PB-AC6 (which added only the
`OpponentCastNSpells` note); flagged because it is a live approximation in the reviewed set.

---

## MEDIUM

### M1 — Kaito Shizuki: −2 authored as a no-op `Effect::Nothing` loyalty ability
`crates/engine/src/cards/defs/kaito_shizuki.rs:48-52`

The +1 is authored correctly (mandatory-unless-attacked discard — verified against oracle
"Draw a card. Then discard a card unless you attacked this turn"; end-step phase-out and −7
are correctly comment-only). But the −2 is present as
`LoyaltyAbility { cost: Minus(2), effect: Effect::Nothing }`. A player can activate −2, pay
2 loyalty, and get nothing — a dud ability (KI-10 analog). The −7 was handled correctly
(fully omitted as a comment); the −2 should be handled the same way until `TokenSpec`
supports the "can't be blocked" static (creating a *blockable* 1/1 Ninja would itself be
wrong game state, so the token cannot be approximated either).

Fix: remove the −2 `LoyaltyAbility` and leave it as a comment (matching −7). The task brief
itself states only the +1 should be authored and −2/−7 remain unauthored.

---

## LOW

### L1 — Black Market: oracle_text uses card name instead of "this enchantment"
`crates/engine/src/cards/defs/black_market.rs:13` — def reads "put a charge counter on
Black Market … add {B} for each charge counter on Black Market"; current Scryfall oracle is
"…on this enchantment" (both clauses). Cosmetic (KI-18). Abilities are correct: verified
`AtBeginningOfFirstMainPhase` fires only for the controller when active
(`turn_actions.rs:561` filters `controller != active`), matching "your first main phase";
`WheneverCreatureDies { controller: None }` correctly matches all creatures.

### L2 — Ripples of Undeath: oracle_text materially misstates the cost
`crates/engine/src/cards/defs/ripples_of_undeath.rs:12` (and header) — def reads "you may
pay 1 life. If you do, return a card…"; real oracle is "you may pay **{1} and 3 life**. If
you do, **put** a card from among those cards **into** your hand." No game impact (abilities
are correctly empty — the milled-set target pool is a real gap), but the oracle_text field
is inaccurate (KI-18). The ENGINE-BLOCKED marker itself is valid.

### L3 — Goldspan Dragon: blocked-marker mischaracterizes the Treasure static as an "override"
`crates/engine/src/cards/defs/goldspan_dragon.rs:50-53` — the marker says the static
"replaces the Treasure's own printed mana ability … can override an existing activated
ability's mana output." Goldspan actually grants Treasures an *additional* ability (they
keep their normal "add one mana of any color"). The blocked conclusion is still correct —
there is no primitive to grant a full activated mana ability to a filtered permanent set
(`LayerModification` supports `AddKeyword`, not add-activated-ability) — but the reasoning
should say "grant an additional activated ability to a filtered set," not "override." The
two `create Treasure` trigger halves (WhenAttacks + WhenBecomesTarget{None,false,false}) are
correct and cannot double-trigger (disjoint events); the split is behaviorally sound.

### L4 — Land Tax: "you may search … up to three" modeled as a mandatory auto-search
`crates/engine/src/cards/defs/land_tax.rs` — three sequential `SearchLibrary` calls
implement "up to three" and the intervening-if is correct, but the "**you may**" is not
modeled (the search/shuffle is forced). This is the established engine convention
(farhaven_elf, dark_petition) and is called out in the file comment; noting only for
completeness. Minor game-state deviation (forced library shuffle even when the controller
would decline).

### L5 — Minor oracle-text/reminder nits
- `kaito_shizuki.rs:20` — "if Kaito Shizuki entered this turn" vs oracle "if Kaito entered
  this turn" (self-reference name expansion). Cosmetic; the clause is unauthored anyway.
- `black_market_connections.rs:15` — omits the "(It is every creature type.)" changeling
  reminder text. Reminder-text omission is acceptable; noted only for completeness.

---

## Cards verified CLEAN (oracle + DSL correct, markers valid)

Fully authored:
- **Searslicer Goblin** — end-step trigger, `intervening_if: YouAttackedThisTurn`, 1/1 red
  Goblin token. {1}{R}, 2/1. Correct.
- **Chart a Course** — draw 2 then mandatory-unless-attacked discard via `Conditional`
  (if_false = discard). {1}{U}. Correct.
- **Bloodsoaked Champion** — CantBlock + graveyard-zone activated ability with
  `activation_condition: YouAttackedThisTurn`, `activation_zone: Graveyard`. {B}, 2/1.
  Correct.
- **Idol of Oblivion** — {T} draw gated by `CreatedATokenThisTurn`; {8},{T},Sac for 10/10
  Eldrazi. {2}. Correct.
- **Dark Petition** — tutor to hand + shuffle, then `Conditional{SpellMastery}` adds
  `mana_pool(0,0,3,0,0,0)` = BBB (WUBRGC order correct). {3}{B}{B}. "two or more instant
  and/or sorcery" union is engine-side (`Condition::SpellMastery`). Correct.
- **Black Market** — (abilities correct; oracle_text nit is L1).
- **Goldspan Dragon** — becomes-target half correct (marker nit is L3).

Partial/blocked with valid markers (nothing approximated):
- **Bonecrusher Giant // Stomp** — trigger fixed to `WhenBecomesTarget{None,false,false}`
  (matches "becomes the target of a spell", any controller, spells only). Removing the
  `EachOpponent` damage was **correct**: verified `Effect::DealDamage` takes only
  `EffectTarget`, and `EffectTarget` has no triggering-player/"that spell's controller"
  variant (only `PlayerTarget` has `TriggeringPlayer`/`ControllerOf`, which `DealDamage`
  can't consume). "That spell's controller" is genuinely inexpressible → omission correct.
  Stomp adventure face (2 dmg any target; prevention-removal TODO valid) correct.
- **Venerated Rotpriest** — confirmed blocked. `Effect::AddCounter` execution
  (`effects/mod.rs:2192-2217`) only handles `ResolvedTarget::Object` and silently ignores
  players, so no poison counter can be given to a *player* (despite the enum's misleading
  "on a permanent or player" doc). Also no `TargetRequirement::TargetOpponent`. Toxic 1
  authored. Marker valid.
- **Raiders' Wake** — first ability authored (`WheneverOpponentDiscards` →
  `LoseLife{TriggeringPlayer}`), correct. Raid half correctly omitted (no
  `TargetRequirement::TargetOpponent`; `TargetPlayer` would allow self-target = wrong).
- **Alesha, Who Laughs at Fate** — first strike + attack `+1/+1` counter authored. Raid
  reanimation blocked: `TargetFilter::max_cmc` is a fixed `u32`, no dynamic comparison to
  Alesha's counter-boosted power. Marker valid.
- **Black Market Connections** — `abilities: vec![]`; "choose one or more" modal on a
  *triggered* ability is genuinely unsupported (contrast H1, where the sibling card wrongly
  authored a dud modal). Correct handling.
- **Florian, Voldaren Scion** — first strike authored; postcombat-main impulse-look blocked
  (no opponents'-life-lost EffectAmount, no impulse-play). Marker valid.
- **Tymna the Weaver** — Lifelink + Partner authored; postcombat draw blocked (no
  "opponents dealt combat damage this turn" tracker/EffectAmount). Marker valid.
- **Scalelord Reckoner** — Flying authored; trigger expressible
  (`WhenBecomesTarget{Some(Dragon you control), by_opponent:true, include_abilities:true}`)
  but effect can't scope "destroy target nonland permanent **that player** controls" to the
  triggering opponent. Marker valid.
- **Flowerfoot Swordmaster** — Offspring keyword + Offspring cost dual-def present (KI-6
  satisfied). Valiant blocked: `WhenBecomesTarget.by_opponent` is a bool (false=any,
  true=opponent-only), no you-control-only scope. Marker valid.
- **Minas Tirith** — Legendary supertype present; ETB-tapped-unless-legendary-creature
  authored as `EntersTapped` + `unless_condition: ControlLegendaryCreature` (matches oracle
  exactly); {T}:{W} authored. Third ability blocked: needs count-based
  `AttackedWithNCreatures(2)`; `YouAttackedThisTurn` (bool) can't distinguish 1 vs 2
  attackers. Marker valid.
- **Battle Cry Goblin** — {1}{R} Goblin anthem+haste authored via two
  `ApplyContinuousEffect`; Pack tactics blocked (needs total-power-≥6-this-combat
  condition; `YouAttackedThisTurn` bool insufficient). Marker valid.

## Integration tests (`crates/engine/tests/pb_ac6_card_integration.rs`)

All 6 assert what their names claim and none passes for an incidental reason:
- All specs routed through `enrich_spec_from_def` (naked `ObjectSpec::card()` would make
  casts free / activations return `InvalidAbilityIndex`).
- Each test exercises **both** branches of its condition (attacked/not, token/none,
  mastery/not, more-lands/equal).
- Test 3 (Bloodsoaked) sets the raid flag via a real `DeclareAttackers` on a separate
  "Raider", not a manual flag poke.
- Test 4 (Idol) explicitly panics on `InvalidAbilityIndex` to prevent a false pass from an
  unenriched spec — the strongest guard in the file.
- Test 6 (Land Tax) documents OOS-AC6-2 (generic upkeep sweep queues then evaluates
  intervening-if at resolution) and asserts observable state rather than stack contents —
  a pre-existing sweep behavior, not a PB-AC6 regression.

No test findings.
