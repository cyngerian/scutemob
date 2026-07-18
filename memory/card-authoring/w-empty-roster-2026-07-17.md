# W-EMPTY roster — the empty `vec![]` (Inert-marked) bucket

**Task**: `scutemob-96` — W-EMPTY (campaign plan §3).
**Date**: 2026-07-17.
**Source of truth**: the 60 defs carrying `completeness: Completeness::inert(...)`.
These are exactly the "empty" bucket of `tools/authoring-report.py`
(`MARKER_BUCKETS = {"inert": "empty", ...}`). Cross-checked against the compiled
registry: `grep -rl "Completeness::inert" crates/card-defs/src/defs` = **60**, and
`authoring-report.py` reports **empty 60**. Marker taxonomy is authoritative (SR-2);
the count is not derived from an `abilities: vec![]` source regex (the
`mana_abilities: vec![]` trap).

## Headline

The campaign plan (2026-05-16) estimated **~110** authorable empty placeholders.
That estimate predates the **marker sweep** (`scutemob-88`, 2026-07-16), which
audited every non-Complete marker *after* the entire PB-AC chain shipped and
**already drained this bucket** — it upgraded the then-authorable empties to
Complete and rewrote every remaining inert note to name the *real* current blocker.
So the empty bucket today is 60, and it is overwhelmingly the genuinely-blocked
residual, not a backlog of easy authoring.

**Verification method (per SR-34/36 + `feedback_verify_full_chain`)**: I did not
trust the notes on their face. For every note claiming a primitive is missing I
walked the full chain against `crates/card-types` + `crates/engine`. Confirmations:
`DelayedTrigger` exists but only with object-return/sacrifice/exile actions (no
draw, no conditional-destroy) — benefactors_draught/berserk/pact stay blocked; no
`Condition::CreatureDiedThisTurn` / `OpponentLostLifeThisTurn` / discarded-this-turn
variant — tragic_slip/braids/bloodchief/tinybones stay blocked; no `optional` field
on `AbilityDefinition::Triggered` and no `MayDo`/`OptionalEffect` wrapper —
blood_seeker stays blocked. Conversely I confirmed the primitives the three
authorable cards need all exist **and are ungated**: `MayPayThenEffect` (ungated,
CR 118.12 — distinct from the gated `MayPayOrElse`), `EffectAmount::PowerOfSacrificedCreature`,
`EffectAmount`/`DrawCards`/`SetNoMaximumHandSize`, `ReplacementModification::EntersTappedUnlessPayLife(u32)`,
`AbilityDefinition::Fuse` + `KeywordAbility::Fuse`, and the four Turn layer mods
(`RemoveAllAbilities`/`SetColors`/`SetCreatureTypes`/`SetPowerToughness`).

## Disposition summary

| Disposition | Count |
| --- | ---: |
| **Authorable now** (author Complete this wave) | **3** |
| Blocked — genuine current DSL/engine gap (note refined, stays inert) | 57 |
| **Total inert** | **60** |

## Authorable now (3)

| Card | Faces | Primitives (all verified present + ungated) | Reference defs |
| --- | --- | --- | --- |
| `turn.rs` (Turn // Burn) | Instant // Instant, Fuse | Turn: `Spell` → `Sequence` of 4 `ApplyContinuousEffect` (`RemoveAllAbilities` L6, `SetColors({Red})` L5, `SetCreatureTypes({Weird})` L4, `SetPowerToughness{0,1}` L7b), `UntilEndOfTurn`; Burn: `AbilityDefinition::Fuse{name:"Burn", cost {1}{R}, DealDamage 2 to any target}`; `KeywordAbility::Fuse` | `connive.rs`, `wear_tear.rs` (Fuse); `revitalizing_repast.rs` (continuous-effect on target) |
| `disciple_of_freyalise.rs` (Disciple of Freyalise // Garden of Freyalise) | Creature — Elf Druid {3}{G}{G}{G} // Land | Front: ETB `MayPayThenEffect{cost: Sacrifice(another creature), then: Sequence[GainLife(PowerOfSacrificedCreature), DrawCards(PowerOfSacrificedCreature)]}`. Back (Garden of Freyalise): `EntersTappedUnlessPayLife(3)` + `{T}: Add {G}` | `lifes_legacy.rs`, `greater_good.rs` (PowerOfSacrificedCreature); `revitalizing_repast.rs` (MDFC land back) |
| `sea_gate_restoration.rs` (Sea Gate Restoration // Sea Gate, Reborn) | Sorcery {4}{U}{U}{U} // Land | Front: `Sequence[DrawCards(HandSize+1, locked before draw, CR 608.2h), SetNoMaximumHandSize{Controller}]`. Back (Sea Gate, Reborn): `EntersTappedUnlessPayLife(3)` + `{T}: Add {U}` | `wrenn_and_seven.rs`, `ancient_silver_dragon.rs` (SetNoMaximumHandSize); `revitalizing_repast.rs` (MDFC land back) |

Exact face oracle text (from `cards.sqlite` `card_faces`, the authoritative DB):
- **Turn** {2}{U}: "Until end of turn, target creature loses all abilities and becomes a red Weird with base power and toughness 0/1." + Fuse
- **Burn** {1}{R}: "Burn deals 2 damage to any target." + Fuse  *(no lifegain — the marker note was correct)*
- **Disciple of Freyalise** {3}{G}{G}{G}: "When this creature enters, you may sacrifice another creature. If you do, you gain X life and draw X cards, where X is that creature's power."
- **Garden of Freyalise** (Land): "As this land enters, you may pay 3 life. If you don't, it enters tapped. / {T}: Add {G}."  *(← EntersTappedUnlessPayLife(3), NOT plain enters-tapped — the def note said only "Land")*
- **Sea Gate, Reborn** (Land): "As this land enters, you may pay 3 life. If you don't, it enters tapped. / {T}: Add {U}."

## Blocked (57) — grouped by real current blocker

Every note below was re-verified against the current tree; each remains a genuine
gap. These keep their inert marker (note refined where it was imprecise or
stale-framed). They are **not** authored wrong (W6 policy).

### Interactive choice / optional election (M10+)
- `blood_seeker` — optional no-cost "you may have that player lose 1 life" trigger; no `optional` field on `Triggered`, `Effect::Choose` is a gated stub. *(Note previously led with "Authorable now" — misleading; refined this wave to name the blocker as the leading clause.)*
- `abundance` — draw-replacement with land/nonland player choice + reveal-until-found + bottom-reorder.
- `sylvan_library` — draw-step replacement (draw two extra) then choose two to pay 4 life or put back.
- `torment_of_hailfire` — two-option cost election (sacrifice a nonland OR discard); `MayPayOrElse` takes one static Cost and always applies `or_else`.
- `season_of_gathering` — Phyrexian-mana mode budget (repeatable modes).
- `valakut_awakening` — variable-count card selection from hand feeding a draw count (front) + MDFC back.
- `turntimber_symbiosis` — optional single-card pick from a looked-at set (RevealAndRoute has no cap / no "you may") + MV-conditional counters + MDFC back.

### "Choose a color" → protection/mana of chosen color
- `mother_of_runes` — protection from color of your choice (interactive choice + dynamic ProtectionQuality grant).
- `sejiri_shelter` — protection from chosen color (front) + MDFC back (Sejiri Glacier).
- `throne_of_eldraine` — produced mana restricted to chosen color / monocolored (no `ManaRestriction` variant; PB-Q2/Q5).

### Draw / library replacement effects
- `laboratory_maniac` — empty-library-draw → win replacement.
- `teferis_ageless_insight` — "if you'd draw, draw twice" replacement with draw-step exception.
- `out_of_the_tombs` — WouldDraw replacement beyond `SkipDraw` (return a creature or lose the game). Same class as Lab Maniac.
- `mox_diamond` — ETB "discard a land or put this in graveyard" replacement (no such `ReplacementModification`).
- `chrome_mox` — ETB Imprint (exile a colored card from hand) + add mana of exiled card's colors.

### Delayed triggers / timing restrictions
- `benefactors_draught` — delayed "draw a card at the next end step" (no draw `DelayedTriggerAction`; only object return/sac/exile exist — verified).
- `berserk` — TimingRestriction has only SorcerySpeed/AnyTime ("before combat damage step" inexpressible) + delayed conditional destroy.
- `pact_of_negation` — {0} counter + delayed upkeep "pay {3}{U}{U} or lose the game."

### Missing "this turn" game-state conditions
- `tragic_slip` — morbid `Condition::CreatureDiedThisTurn` (verified absent).
- `bloodchief_ascension` — "an opponent lost 2+ life this turn" + quest-counter chassis.
- `braids_arisen_nightmare` — end-step sacrifice choice + type-matched opponent sacrifice + per-opponent conditional draw.
- `tinybones_trinket_thief` — "an opponent discarded a card this turn" condition.
- `keen_eyed_curator` — "cards exiled with this permanent" association on GameObject + distinct-card-type count condition.
- `scavenging_ooze` — no condition testing "the exiled card was a creature card" to gate the counter+lifegain rider (the {G} exile itself is expressible).

### Trigger scope / filter gaps
- `duelists_heritage` — global "whenever one or more creatures attack" (TriggerEvent is controller-scoped only).
- `dragon_tempest` — `EffectFilter::TriggeringCreature` on a continuous effect + entering Dragon as the damage source.
- `seedborn_muse` — no trigger for "each other player's untap step" (effect half `UntapAll{filter}` exists).
- `nadiers_nightblade` — no trigger for "a token you control leaves the battlefield."
- `serpents_soul_jar` — death-trigger that exiles the dying object (Elf-dies-exile) + reanimation clauses.
- `esper_sentinel` — dynamic cost = this creature's power (EffectAmount-valued cost) + per-opponent once-per-turn (global `once_per_turn` would be wrong in multiplayer).
- `rhythm_of_the_wild` — confer uncounterability on your other spells (self-only field today) + `EffectFilter` nontoken variant for the Riot grant.

### Zone / graveyard-target effects
- `boggart_trawler` — "exile target player's graveyard" mass GY-zone exile (ExileAll is battlefield-only) + MDFC back (Boggart Bog).
- `animate_dead` — reanimation Aura enchanting a graveyard card (no graveyard `EnchantTarget`) + enchant-target change on resolution.

### Room / Door mechanic (CR 725/726)
- `bottomless_pool` — Room door-unlock chassis (body expressible).
- `funeral_room` — Room door-unlock chassis (body expressible).
- `walk_in_closet` — Room door chassis (StaticPlayFromGraveyard exists behind it).

### Flip cards (CR 711)
- `rune_tail_kitsune_ascendant` — no flip-face model / flip action / flipped-face static.

### Modal / target / cost gaps
- `akromas_will` — commander-gated `max_modes` widening (condition-gated ModeSelection) + protection-quality grant. Shares blocker with jeska's_will.
- `naya_charm` — mode 3 "tap all creatures target player controls" (TapPermanent targeting a player-filtered set).
- `return_to_dust` — "if cast during your main phase" conditional second exile target.
- `tear_asunder` — Kicker changes the legal target set (artifact/enchant vs nonland permanent); `targets` is fixed at authoring.
- `keep_safe` — counter a spell "that targets a permanent you control" (spell-target filter).
- `reprieve` — return target spell to owner's hand (Stack→Hand bounce, not counter) + draw rider.
- `tibalts_trickery` — hard counter + random mill + exile-until-nonland + free cast for controller.
- `commit` — `LibraryPosition` has no Nth-from-top ("second from the top") + Memory half unauthored.
- `resculpt` — token must go to controller of the exiled permanent (`CreateToken` always creates for the caster).
- `excise_the_imperfect` — `Effect::Incubate` (Incubator token: transforming token w/ X +1/+1 counters + "{2}: Transform").

### Cost / sacrifice / amount gaps
- `nullmage_shepherd` — cost "tap N other untapped creatures you control" (`Cost::TapCreatures`).
- `entish_restoration` — `SacrificePermanents` lacks a type filter (must sac a Forest specifically) + reveal/put-onto-battlefield clause.
- `quietus_spike` — `EffectAmount` has no half-rounded-up variant ("loses half their life, rounded up"). Deathtouch grant + Equip {3} + the trigger itself are all expressible.

### Combat-relationship / dynamic-power conditions
- `sting_the_glinting_dagger` — first strike while blocking/blocked by a Goblin or Orc (no combat-relationship-to-subtype condition). The other three clauses are expressible.
- `selvala_heart_of_the_wilds` — both abilities need dynamic greatest-power comparisons.

### Chosen-type / designation
- `kindred_discovery` — `ChosenType` designation on GameObject (both triggers key off the chosen type).

### Opening-hand / start-of-game
- `gemstone_caverns` — conditional opening-hand start (not-the-starting-player gate + enters-with-luck-counter payload + "exile a card from your hand" cost). `OpeningHand` is an unconditional marker.

### Granted-ability layer mod
- `malakir_rebirth` — `LayerModification::AddTriggeredAbility` (grant "when this dies, return it tapped") + MDFC back (Malakir Mire).

### Random discard
- `gamble` — `Effect::DiscardAtRandom` does not exist.

## Engine findings

No *new* engine findings were discovered by this triage — every blocker here is
already captured by an existing note and, where it names a primitive gap, by the
marker-sweep engine-findings ledger (`marker-sweep-engine-findings-2026-07-16.md`)
and the standing OOS/EF backlog. The three authorable cards need no engine work.

The recurring high-value gaps this bucket re-confirms (candidates for a future PB,
already known): (a) **optional no-cost "you may" triggered effect** (blood_seeker,
and the interactive-choice cohort) — the single most common blocker here; (b) a
**"this turn" event-history condition** family (morbid, opponent-lost-life,
opponent-discarded); (c) the **Room/Door chassis** (3 cards); (d) **choose-a-color →
protection/mana** (3 cards). None are filed as new EF-W-EMPTY-* because each is
already tracked.
