---
name: DSL Gap Audit (2026-05-16)
description: Refreshed classification of all 1,165 card-def TODOs against the post-LOW-Sweep DSL surface. Supersedes dsl-gap-audit-v2.md (2026-03-22 / 2026-04-19).
type: reference
---

# DSL Gap Audit — 2026-05-16

**Supersedes**: `dsl-gap-audit-v2.md` (last meaningfully refreshed 2026-04-19).
**Ground truth**: `docs/authoring-status.md` regenerated 2026-05-16 (git `08825d09`).
**Scope**: every `// TODO` line in `crates/engine/src/cards/defs/*.rs`.

## Why a refresh was needed

The v2 audit predates the entire `scutemob-22..38` primitive chain and the LOW
Sweep campaign. Those shipped: `Effect::DestroyAndReanimate`, `Effect::PreventNextUntap`,
`TargetFilter.is_blocking/is_tapped/is_untapped`, `ObjectFilter::CreatureControlledByOfSubtype`,
`EffectAmount::CounterCountAtLastKnownInformation`, `EffectAmount::SourcePowerAtLastKnownInformation`,
`pre_death_characteristics` LKI snapshot, and (earlier, but not reflected in v2's
"STILL_BLOCKED" buckets) the **`filter: Option<TargetFilter>` field on the
`WheneverCreatureDies` / `WheneverCreatureEntersBattlefield` / `WheneverPermanentEntersBattlefield` /
`WheneverCreatureYouControlAttacks` / combat-damage trigger conditions** (PB-23 / PB-26 / PB-30 / PB-N).

The net effect: a **large fraction of TODOs that v2 classified STILL_BLOCKED are
now stale** — the primitive shipped but the card def was never re-authored.

## Headline numbers

| Metric | Count |
| --- | ---: |
| Card def files on disk | 1,748 |
| Clean (no TODO, real abilities) | 924 (52.9%) |
| Files with TODO markers | 642 |
| Empty `abilities: vec![]` placeholders | 182 |
| Total TODO lines | 1,165 |
| Plan cards with NO def file | 194 |

### TODO disposition (this audit's verdict)

| Verdict | TODO lines (est.) | Meaning |
| --- | ---: | --- |
| **NOW-EXPRESSIBLE** (stale) | ~330 | Primitive exists today; card just needs re-authoring. No engine work. |
| **ENGINE-BLOCKED** | ~620 | Genuinely needs a new DSL primitive. Grouped into PBs below. |
| **DEFER** | ~215 | Honestly out of scope: hidden-info, digital-only, post-alpha, or cosmetic notes. |

These are line counts; a single card def often carries 2-4 TODO lines, so the
*card* counts are smaller. Roughly: ~210 cards are pure NOW-EXPRESSIBLE, ~430 cards
are ENGINE-BLOCKED (many also partially expressible), ~110 cards are DEFER.

---

## The current DSL surface (what actually exists)

Read directly from `crates/engine/src/cards/card_definition.rs` on 2026-05-16.
Card authors and PB planners must trust THIS list, not v2's gap buckets.

**`TriggerCondition`** (rich — most v2 "missing trigger" gaps are closed):
self ETB/dies/attacks/blocks, `WheneverCreatureDies { controller, exclude_self, nontoken_only, filter }`,
`WheneverCreatureEntersBattlefield { filter, exclude_self }`,
`WheneverPermanentEntersBattlefield { filter, exclude_self }`,
`WheneverCreatureYouControlAttacks { filter }`,
`WheneverCreatureYouControlDealsCombatDamageToPlayer { filter }`,
`WhenOneOrMoreCreaturesYouControlDealCombatDamageToPlayer { filter }`,
`WheneverYouCastSpell { during_opponent_turn, spell_type_filter, noncreature_only, chosen_subtype_filter }`,
`WheneverOpponentCastsSpell { spell_type_filter, noncreature_only }`,
`WheneverPlayerDrawsCard`, `WheneverYouGainLife`, `WheneverYouDrawACard`,
`WheneverYouSacrifice { filter, player_filter }`, `WheneverYouDiscard`, `WheneverOpponentDiscards`,
`WheneverYouAttack`, `WhenLeavesBattlefield`, `WhenYouCastThisSpell`, `WhenDealtDamage`,
`WhenSelfBecomesTapped`, `WhenTappedForMana`, `WhenBecomesTargetByOpponent`,
upkeep/endstep/combat begin, plus mechanic-specific (surveil/connive/investigate/mutate/etc.).

**`Effect`** (~95 variants): includes `GainControl`, `ExchangeControl`, `AdditionalLandPlay`,
`Effect::Proliferate`, `UntapPermanent`, `PreventNextUntap`, `DestroyAndReanimate`,
`GrantPlayerProtection`, `PutLandFromHandOntoBattlefield`, `RevealAndRoute`, `Flicker`,
`ExileWithDelayedReturn`, `ExtraTurn`, `BecomeCopyOf`, `CreateTokenCopy`, `MayPayOrElse`,
`AdditionalCombatPhase`, `CopySpellOnStack`, `ChangeTargets`, `RegisterReplacementEffect`,
`SacrificePermanents`, `AttachEquipment`, all mana variants.

**`TargetFilter`** (rich): `legendary`, `nonbasic`, `basic`, `has_card_types` (OR),
`has_subtypes` (OR), `exclude_subtypes`, `min/max_power`, `max_toughness`, `min/max_cmc`,
`colors` / `exclude_colors`, `is_token` / `is_nontoken`, `is_attacking` / `is_blocking`,
`is_tapped` / `is_untapped`, `has_counter_type`, `has_name`, `has_chosen_subtype`, `exclude_self`.

**`Cost`**: `Mana`, `Tap`, `SacrificeSelf`, `ExileSelf`, `Sacrifice(TargetFilter)`,
`PayLife`, `DiscardCard`, `DiscardSelf`, `Forage`, `Sequence`, `RemoveCounter`.
**`SpellAdditionalCost`**: `SacrificeCreature`, `SacrificeLand`, `SacrificeArtifactOrCreature`,
`SacrificeSubtype`, `SacrificeColorPermanent`.

**`Condition`** (~40 variants): includes `YouControlNOrMoreWithFilter { count, filter }`,
`OpponentLifeAtMost`, `IsYourTurn`, `WasCast`, `XValueAtLeast`, devotion, dungeon,
delirium, ascend, `And/Or/Not` combinators.

**`ContinuousEffectDef`** carries an `Option<Condition>` — conditional statics
("as long as X") are fully expressible (this closes v2's G-03 / G-13 / G-18).

**`EffectTarget::TriggeringCreature`** exists — closes v2's G-12.
**`ActivationZone::Graveyard`** + **`TriggerZone::Graveyard`** exist — closes v2's G-11.
**`SpellCostModifier`** with per-color reduction exists — closes v2's G-27.

---

## Stale-TODO clusters (NOW-EXPRESSIBLE — re-author, no engine work)

The single biggest finding of this refresh: large blocks of TODOs cite primitives
that **shipped months ago**. Sample verification (read the actual card files):

| Stale cluster | Example cards | TODO text claims… | Reality |
| --- | --- | --- | --- |
| Subtype-filtered ETB triggers | `ganax_astral_hunter`, `lathliss_dragon_queen` | "subtype-filtered ETB trigger not in DSL" | `WheneverCreatureEntersBattlefield { filter, exclude_self }` exists |
| Subtype-filtered death triggers | `crossway_troublemakers`, `pashalik_mons`, `omnath_locus_of_rage` | "WheneverCreatureDies lacks subtype filter" | `WheneverCreatureDies { filter, … }` exists |
| "Creature you control attacks" triggers | `shared_animosity`, `hellrider` | "WheneverCreatureYouControlAttacks does not exist" | It exists, with a `filter` field |
| GainControl effects | 9 files w/ TODO + "gain control" | v2 G-04 "GainControl effect missing" | `Effect::GainControl` + `Effect::ExchangeControl` exist |
| Additional-land / extra land drops | bounce-land-adjacent, `wayward_swordtooth` | v2 G-07 "play-lands permission missing" | `Effect::AdditionalLandPlay` + `PutLandFromHandOntoBattlefield` exist |
| Conditional statics ("as long as X") | v2 G-03/G-13/G-18 cohort | "conditional grant not in DSL" | `ContinuousEffectDef.condition` + `Condition::YouControlNOrMoreWithFilter` exist |
| Triggering-object targeting | v2 G-12 cohort | "EffectTarget::TriggeringPermanent missing" | `EffectTarget::TriggeringCreature` exists |
| Graveyard-zone activated abilities | v2 G-11 cohort | "activation_zone missing" | `ActivationZone::Graveyard` exists |
| `Cost::PayLife` / `Cost::Sacrifice` | per v2 STALE table | "Cost enum lacks variant" | Both exist |
| Cost-reduction-per-legendary | channel lands | v2 G-27 | `SpellCostModifier` with colored reduction exists |
| `EffectAmount` LKI variants | conclave-mentor-like dies-triggers | "power on death not knowable" | `SourcePowerAtLastKnownInformation` + `CounterCountAtLastKnownInformation` ship |

**Estimated NOW-EXPRESSIBLE: ~330 TODO lines across ~210 card files.** These need
no engine work — only re-authoring (look up oracle text, write the DSL, delete the
TODO). They are the campaign's free win.

> **Caveat — verify per card.** The v2 audit's per-card NOW-EXPRESSIBLE table (its
> Phase-1 list) was largely actioned in F-4 / BF-S1..S4. The stale TODOs remaining
> today are mostly the *subtype-filtered-trigger* and *conditional-static* cohorts
> that shipped AFTER v2's last refresh. A re-authoring worker must still confirm
> the construct against `card_definition.rs` before deleting each TODO — staleness
> is a strong prior, not a guarantee.

---

## Engine-blocked gap buckets

Genuine gaps, grouped by the primitive they need. Card counts are raw file counts
mentioning the theme; per the project's known 2-3x overcount bias, treat the
"realistic yield" column as the planning number.

| Gap ID | Bucket | Missing primitive | TODO lines | Realistic card yield | Effort |
| --- | --- | --- | ---: | ---: | --- |
| GN-01 | CDA / count-based dynamic P/T (residual) | `LayerModification::ModifyBoth` taking `EffectAmount` (not `i32`); `EffectAmount` count variants — `AttackingCreatureCount`, `TappedCreatureCount`, `HandSize`, power-based token count | ~36 | 12-16 | L |
| GN-02 | Optional "you may pay / you may" riders | `Effect::MayPayOrElse` exists for counter-tax; needs a general **optional-cost wrapper** on triggered effects + optional token/sacrifice | ~31 | 14-18 | M |
| GN-03 | Untap-step / untap-all primitives | `Effect::UntapAll { filter }`; `TriggerCondition::WheneverPermanentUntaps`; "doesn't untap" static; "untap during each other untap step" static (Seedborn Muse) | ~20 | 10-13 | M |
| GN-04 | Counter-unless-pays | `Effect::CounterUnlessPays { cost }` (distinct from `MayPayOrElse` — caster-side vs controller-side); Ferocious/conditional variants | ~9 | 6-8 | S |
| GN-05 | "up to N" optional targets | `TargetRequirement` optional-target / `UpToN` variant; per-mode optional targets | ~15 | 10-14 | M |
| GN-06 | Per-mode target requirements (modal spells) | per-mode `TargetRequirement` attached to `ModeSelection` entries | ~5 | 8-12 | M |
| GN-07 | No-maximum-hand-size | `GameRestriction::NoMaximumHandSize` player flag | ~7 | 5-7 | S |
| GN-08 | Wheel / mass shuffle-hand-draw | `Effect::WheelHand` (shuffle hand into library, draw that many) | ~2 | 3-4 | S |
| GN-09 | Warp alt cost | `AltCostKind::Warp` + `KeywordAbility::Warp` | ~4 | 3-4 | S |
| GN-10 | Transmute | `KeywordAbility::Transmute` + tutor-by-MV activation from hand | ~2 | 2-3 | S |
| GN-11 | Exert | `KeywordAbility::Exert` + skip-next-untap tracking | ~2 | 2-3 | S |
| GN-12 | Per-mana-spent triggers / linked mana | "when you spend this mana to cast X" trigger; mana-origin tracking | ~6 | 4-6 | M |
| GN-13 | Cost::ExileFromHand (pitch/free-cast) | `Cost::ExileFromHand` / pitch alt-cost (Elvish Spirit Guide, Force-style) | ~5 | 6-9 | M |
| GN-14 | First-main / postcombat-main phase triggers | `TriggerCondition::AtBeginningOfFirstMainPhase` / `AtBeginningOfPostcombatMain` | ~5 | 4-6 | S |
| GN-15 | d20 roll + variable multi-target reanimation | `Effect::RollD20` + outcome-tiered branching; multi-target GY-return with total-power cap | ~4 | 3-5 | M |
| GN-16 | "becomes target" / valiant triggers | `TriggerCondition::WhenBecomesTarget { by }` (own + opponent), batch | ~4 | 4-6 | S |
| GN-17 | Attack-restriction / must-attack / can't-attack-owner | `StaticRestriction` variants for forced attack, "can't attack its owner", "can't be sacrificed" | ~8 | 6-9 | M |
| GN-18 | Opponent-action tracking conditions | `Condition::YouAttackedThisTurn`, `OpponentCastNSpells`, `OpponentControlsMoreLandsThanYou`, `SpellMastery`, `CreatedATokenThisTurn` | ~12 | 10-14 | M |
| GN-19 | Token doubling / token replacement | `ReplacementModification::TokenDoubling`; "create one of each instead" | ~5 | 5-7 | M |
| GN-20 | Counter-placed trigger + once-per-turn limiter | `TriggerCondition::WhenCounterPlaced`; generic `once_per_turn` flag on triggered abilities | ~10 | 12-16 | M |
| GN-21 | Discarded-card-type-filtered trigger | filter on `WheneverYouDiscard` / `WheneverOpponentDiscards` (Madness-adjacent) | ~3 | 4-6 | S |
| GN-22 | Type-changing / "loses all abilities" one-shots & statics | `Effect::LoseAbilities`, `Effect::SetCreatureTypes`, layer-4 type override one-shot (Frodo subtype-swap, Darksteel Mutation) | ~6 | 8-12 | M |
| GN-23 | Spell-subtype filter (Aura/Equipment/Vehicle spells) | `spell_type_filter` extension to carry subtypes, not just `CardType` | ~3 | 5-8 | S |
| GN-24 | Instant-speed planeswalker loyalty / "activate any turn" | timing flag on `LoyaltyAbility` | ~2 | 2-3 | S |
| GN-25 | Win-condition effects | `Effect::WinGame { condition }` (devotion ≥ library, artifact count, etc.) | ~3 | 3-5 | S |
| GN-26 | Glimmer / death-return-as-enchantment, Earthbend, misc keyword actions | one-offs — bundle or defer | ~4 | 2-4 | S |
| GN-27 | Multi-search constraints | `SearchLibrary` "find up to N different-named cards" (Tiamat, up-to-two tutors) | ~4 | 4-6 | M |
| GN-28 | Hybrid-cost filter mana (filter lands) | multi-output `AddManaFilterChoice` with hybrid-cost activation (v2 G-10) | ~7 | 7-9 | M |

Long tail: roughly 60-80 further TODO lines are genuine but singleton gaps (one
card, one bespoke clause). These are not worth a PB each — fold into a final
**"misc gaps" PB** or accept as DEFER.

---

## Empty `abilities: vec![]` defs (182 files)

Spot-checked ~20 files. The empties split roughly:

- **~55-65% authorable now** — the card is a placeholder that simply was never
  written; its abilities are bread-and-butter. Examples: `beetleback_chief`
  (ETB create two tokens), `beast_within` (destroy + opponent makes a token),
  `bastion_of_remembrance` (ETB token + dies-drain — now expressible),
  `avenger_of_zendikar` (ETB token-per-land + landfall counter),
  `bitterblossom` (upkeep lose-life + token), `boggart_trawler` (ETB exile GY).
- **~25-30% engine-blocked** — empty *because* the card hits a real gap.
  Examples: `akromas_will` (modal commander-conditional mass grant — GN-06 + grants),
  `berserk` (double-power + delayed destroy), `alela_cunning_conqueror` (covered
  triggers but token-batch nuance), `abundance` (draw-replacement).
- **~10% DEFER** — genuinely hard (interactive / hidden-info).

**Estimate: ~110 of the 182 empties are authorable now**, ~50 engine-blocked,
~20 defer. The empties are higher-yield-per-file than TODO files because an empty
def is one whole card unlocked, not one clause.

---

## The 194 missing card-def files

Judged by group + sampling `docs/authoring-status-missing.txt`:

| Group | Missing | Authorable now (est.) | Notes |
| --- | ---: | ---: | --- |
| modal-choice | 37 | ~15 | Many blocked: play-from-top, pitch alt-cost, copy-target-spell, gain-control, extra-turn. ~22 need GN-06/GN-13/etc. |
| attack-trigger | 28 | ~22 | Subtype-filtered attack triggers now expressible (stale-cluster). A few (Winota, Klauth) need count/conditional work. |
| activated-tap | 25 | ~20 | Mostly standard tap-for-effect (Arbor Elf, Stoneforge Mystic, Bloom Tender). A few (Scroll Rack, The Chain Veil) are gnarly. |
| other | 23 | ~12 | Heterogeneous — Coat of Arms (CDA-ish anthem), Patriarch's Bidding, Wheel of Fortune adjacents. Mixed. |
| activated-sacrifice | 16 | ~12 | Sac-cost abilities mostly expressible (`Cost::Sacrifice`). Jeweled Lotus / Lotus Petal trivial. |
| body-only | 15 | ~6 | DFC transform cards (Bloodline Keeper, Fable, Westvale Abbey) — transform engine exists but each is bespoke. |
| untap-phase | 12 | ~2 | Mostly engine-blocked: extra-combat (Aggravated Assault, Relentless Assault), Intruder Alarm, Wilderness Reclamation — GN-03. |
| token-create | 10 | ~7 | Scute Swarm (landfall copy-self), Titania — a few need count/copy nuance. |
| static-enchantment | 8 | ~3 | Door of Destinies / Coat-of-Arms-style chosen-type anthems — chosen-type designation gaps. |
| discard-effect | 7 | ~4 | Wheel of Fortune / Windfall need GN-08. Necropotence is bespoke. |
| draw | 6 | ~4 | Mostly standard. |
| sacrifice-outlet | 5 | ~4 | `Cost::Sacrifice` covers most. |
| other singletons | 5 | ~3 | — |

**Estimate: ~115 of 194 missing cards authorable now**, ~65 engine-blocked, ~14 defer.

---

## DEFER bucket (honest out-of-scope)

- Interactive / hidden-info choices that the deterministic engine can only
  approximate (e.g. "look at face-down creatures", opening-hand reveals,
  free-form "any number of" piles) — ~30 lines. Many are *informational* and the
  card already functions; the TODO is a fidelity note, not a blocker.
- Cosmetic / style TODOs (subtype ordering, comment wording, oracle-text
  paraphrase) — ~40 lines. Sweep opportunistically; not campaign-critical.
- Genuinely post-alpha: `Banding` (already deferred), highly bespoke planar /
  un-card mechanics — handful.
- Singleton mechanic one-offs not worth a primitive (Earthbend, Glimmer-as-
  enchantment) unless a cluster forms — fold into GN-26 or defer.

**Estimate ~215 TODO lines / ~110 cards.** Note a chunk of these are non-blocking
fidelity notes on already-functioning cards.

---

## Cross-check against v2

| v2 bucket | v2 said | 2026-05-16 status |
| --- | --- | --- |
| G-01 CDA dynamic P/T | STILL_BLOCKED, 14 | Partially shipped (PB-28 `CdaPowerToughness`); residual = GN-01 |
| G-03 conditional static grant | STILL_BLOCKED, 18 | **CLOSED** — `ContinuousEffectDef.condition` (PB-24/25) |
| G-04 GainControl | STILL_BLOCKED, 11 | **CLOSED** — `Effect::GainControl` / `ExchangeControl` (PB-32) |
| G-07 play-lands / extra land | STILL_BLOCKED, 6 | **CLOSED** — `AdditionalLandPlay` / `PutLandFromHandOntoBattlefield` (PB-K/32) |
| G-08 UntapAll | STILL_BLOCKED, 4 | Still open — GN-03 |
| G-09 bounce-land ETB | STILL_BLOCKED, 14 | **CLOSED** — `MoveZone` to Hand on ETB trigger (PB-32 era) |
| G-10 hybrid filter mana | STILL_BLOCKED, 7 | Still open — GN-28 |
| G-11 GY activated abilities | STILL_BLOCKED, 8 | **CLOSED** — `ActivationZone::Graveyard` |
| G-12 triggering-object target | STILL_BLOCKED, 12 | **CLOSED** — `EffectTarget::TriggeringCreature` |
| G-18 count-threshold static | STILL_BLOCKED, 10 | **CLOSED** — `Condition::YouControlNOrMoreWithFilter` |
| G-27 cost-reduction-per-legendary | STILL_BLOCKED, 5 | **CLOSED** — `SpellCostModifier` |
| G-30 Warp | STILL_BLOCKED, 1 | Still open — GN-09 |
| G-21 trigger doubling (death) | STILL_BLOCKED, 2 | `TriggerDoubling` exists; verify death-filter coverage |
| Subtype-filtered triggers | (not a v2 bucket) | Shipped post-v2; ~19 stale TODOs still cite the old gap |

**Takeaway**: 9 of v2's top-10 "highest-impact gaps" are now closed. The work
left is (a) re-author the stale cohort and (b) ~25 smaller, well-scoped gaps.
