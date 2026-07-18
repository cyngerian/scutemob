# W-MISS Review — Batch 3 (10 cards)

**Reviewed**: 2026-07-17
**Cards**: 10
**Findings**: 0 HIGH, 0 MEDIUM, 1 LOW (informational)
**Verdict**: All 10 correct and legitimately `Complete`. No fixes required. No demotions.

Method: oracle via `mcp__mtg-rules__lookup_card`; every DSL shape traced to its
card-types definition and its engine consumer (matches_filter, resolve_amount,
flatten_cost_into, is_channel path). No gated stub effect (`Effect::Choose`,
`MayPayOrElse`, `AddManaChoice`, `AddManaAnyColor*`) appears anywhere in the batch.

---

## Crucible of Worlds — CORRECT
- {3} Artifact; oracle exact. `StaticPlayFromGraveyard { LandsOnly, condition: None }` (PB-B). Clean.

## Ramunap Excavator — CORRECT
- {2}{G} Creature — Snake Cleric 2/3; oracle exact. Same graveyard-land static as Crucible. Types/P-T correct. Clean.

## Icetill Explorer — CORRECT
- {2}{G}{G} Creature — Insect Scout 2/4; oracle exact (three lines).
- `AdditionalLandPlays { count: 1 }` (extra land), `StaticPlayFromGraveyard(LandsOnly)`,
  landfall `WheneverPermanentEntersBattlefield { filter: Land/You, exclude_self: false }`
  → `MillCards { player: Controller, count: 1 }`. "mill a card" = mill your own = Controller. Correct.

## Bygone Colossus — CORRECT
- {9} Artifact Creature — Robot Giant 9/9; oracle exact.
- Warp modeled with the timeline_culler pattern: `Keyword(Warp)` marker + `AltCastAbility {
  kind: Warp, cost: {3}, details: Warp { costs: vec![], from_graveyard: false } }`.
  No non-mana cost and no graveyard permission — matches Bygone's oracle (hand only), unlike
  Timeline Culler ({B}+2 life, from_graveyard: true). Correct.

## Omnath, Locus of the Roil — CORRECT (borderline, verified)
- {1}{G}{U}{R} Legendary Creature — Elemental 3/3; supertype Legendary present via `full_types`. Oracle exact.
- ETB: `DealDamage { target: DeclaredTarget(0), amount: PermanentCount { filter: {Elemental,
  controller: You}, controller: PlayerTarget::Controller } }`, target `TargetAny`. Damage scope is
  the controller (you), and Omnath counts itself (on battlefield at resolution). Correct.
- Landfall: required `TargetCreatureWithFilter(Elemental/You)`, effect `Sequence[AddCounter(+1/+1 on
  DeclaredTarget0), Conditional(YouControl>=8 lands → DrawCards(Controller,1))]`. Targeted counter +
  conditional draw both modeled; if target illegal at resolution the whole ability fizzles (no draw),
  which matches the single-target ruling. Correct.

## Touch the Spirit Realm — CORRECT (borderline, verified)
- {2}{W} Enchantment; oracle exact (ETB + Channel). Note the real oracle Channel returns "at the
  beginning of the next end step" (not "end of turn" as the brief paraphrased) — the def uses
  `AtNextEndStep`, matching the true oracle.
- ETB: `ExileWithDelayedReturn { WhenSourceLeavesBattlefield, return_to: Battlefield }`, target
  `UpToN { count: 1, TargetPermanentWithFilter(has_card_types: [Artifact, Creature]) }`.
  "up to one" → 0 targets legal. "artifact or creature" → `has_card_types` is OR-semantics
  (effects/mod.rs:8034 "must have at least one of the listed types"). Correct.
- Channel: `Cost::Sequence([Mana {1}{W}, DiscardSelf])`, `activation_zone: None`. Verified the
  `discard_self` flag is set by `flatten_cost_into` (Cost::DiscardSelf) and that the engine's
  is_channel path (abilities.rs:171-187) keys off `cost.discard_self` to permit hand activation,
  so `activation_zone: None` is correct — same pattern as Boseiju/Eiganjo/Otawara channel cards.
  Effect `ExileWithDelayedReturn { AtNextEndStep, Battlefield }` (returns under owner's control =
  default). Correct.

## Flux Channeler — CORRECT
- {2}{U} Creature — Human Wizard 2/2; oracle exact.
- `WheneverYouCastSpell { noncreature_only: true, .. }` → `Effect::Proliferate`. `noncreature_only`
  is wired into the runtime filter in enrich (replay_harness.rs:2470-2475) and is the established
  idiom (esper_sentinel, monastery_mentor, mystic_remora). Correct.

## Dragonmaster Outcast — CORRECT
- {R} Creature — Human Shaman 1/1; oracle exact.
- `AtBeginningOfYourUpkeep` with `intervening_if: YouControl>=6 lands` (CR 603.4), effect
  `CreateToken` 5/5 red Dragon with `Flying`. Token spec correct (creature, Dragon, red, 5/5,
  Flying keyword, not tapped). Correct.

## Revel in Riches — CORRECT (focus card, verified)
- {4}{B} Enchantment; oracle exact.
- Dies trigger: `WheneverCreatureDies { controller: Some(Opponent), exclude_self: false,
  nontoken_only: false, filter: None }` → `CreateToken(treasure_token_spec(1))`. Controller scope =
  opponent-controlled, matching "a creature an opponent controls dies." Treasure spec has count. Correct.
- Win con: `AtBeginningOfYourUpkeep` + `intervening_if: YouControl>=10 {Artifact, subtype Treasure}`
  → `Effect::WinGame`. Correct.

## Scute Swarm — CORRECT (focus card, verified)
- {2}{G} Creature — Insect 1/1; oracle exact.
- Landfall `WheneverPermanentEntersBattlefield { Land/You }` → `Conditional(YouControl>=6 lands →
  CreateTokenCopy { source: Source, except_not_legendary: false, gains_haste: false } ELSE
  CreateToken(1/1 green Insect))`. Copy-of-self on 6+ lands, else 1/1 Insect. Correct.

---

## LOW (informational, no fix needed)
- **L1 (Omnath)**: the ETB `PermanentCount.filter` sets `controller: TargetController::You` *and*
  the `PermanentCount.controller: PlayerTarget::Controller`. `matches_filter` does not read the
  filter's `controller` field (Characteristics carry no controller perspective), so the filter
  clause is a harmless no-op; the actual "you control" restriction comes entirely from the
  `PlayerTarget::Controller` count scope. Result is correct. Could drop the redundant `controller:
  You` from the filter for clarity, but it changes nothing (and would change the struct hash, so
  leave it unless touching the file for another reason).

## Summary
- Cards needing fixes: **none**
- Cards to demote: **none**
- Clean/Complete: all 10 (Crucible of Worlds, Ramunap Excavator, Icetill Explorer, Bygone Colossus,
  Omnath Locus of the Roil, Touch the Spirit Realm, Flux Channeler, Dragonmaster Outcast, Revel in
  Riches, Scute Swarm)
