# W-MISS roster — the authorable missing-file cards

> **UPDATE 2026-07-18 (PB-EF3, `scutemob-103`):** the two engine gaps this wave demoted for
> — EF-W-MISS-10 (targeted attack trigger) and EF-W-MISS-4 (defending-player target) — are now
> CLOSED. Shipped Complete: **Ojutai, Soul of Winter** (`ojutai_soul_of_winter.rs`), **Hellrider**
> (flip), **Raid Bombardment** (new). Still blocked on *other, distinct* primitives (not this
> wave's two gaps): Silumgar (continuous defending-player-scoped -1/-1 → filed **OOS-EF3-1**),
> Brutal Hordechief (force-block + attacker-chosen blocks), Norn's Decree + Karazikar (multi-gap),
> Cunning Rhetoric (defender-side "opponent attacks you" trigger). See
> `memory/card-authoring/w-miss-engine-findings-2026-07-17.md` EF-W-MISS-4/10 (marked CLOSED).

**Task**: `scutemob-97` — W-MISS (campaign plan §3).
**Date**: 2026-07-17.
**Source of truth**: `docs/authoring-status-missing.txt` (regenerated at HEAD `ab42a198`;
**194 cards** still missing on disk) cross-checked card-by-card against the compiled
registry (`ls crates/card-defs/src/defs/`) and the live DSL/engine (`crates/card-types`,
`crates/engine`). Every card's oracle text was pulled from the MTG-rules MCP (authoritative),
and every clause was traced to a concrete ungated primitive per `feedback_verify_full_chain`.

## Headline

The campaign plan (2026-05-16) estimated **~115** authorable missing-file cards, assuming
whole groups (attack-trigger, activated-tap, activated-sacrifice, sacrifice-outlet, most of
draw/token) were bulk-authorable. That estimate is **stale**, for the same reason W-EMPTY's
was (`scutemob-96`): the marker sweep (`scutemob-88`) and the PB-AC chain already shipped the
easy cohorts, and the remaining missing files are dominated by cards whose *headline* clause
is expressible but whose *rider* clause hits a known gap. Verified authorable this wave:
**35 of 194** (plus 2 false-missing that already exist on disk).

The single most common real blocker in this corpus is **`EffectFilter::TriggeringCreature`**
(continuous "it gets +N/+N" / "it gains <keyword>" on the just-attacked creature — 4 attack
cards) and the **"defending player" target gap** (the Hellrider gap — 3 cards). The
body-only bucket is uniformly blocked on a **card-invokable self-transform effect** (11 DFCs).
Gated mana (`AddManaAnyColor`/`AddManaChoice`) and chosen-color/chosen-type machinery block
another large slice.

## Disposition summary

| Disposition | Count |
| --- | ---: |
| **Authored Complete this wave** | **33** |
| Blocked — genuine current DSL/engine gap | 159 |
| Already on disk (false-missing — report name-normalization quirk) | 2 |
| **Total missing (per report)** | **194** |

> **Updates (authoring + review):** Two cards demoted authorable → blocked during the wave:
> - **Misdirection** — oracle requires "target spell **with a single target**"; the DSL's
>   only single-target restriction (`TargetRequirement::TargetSpellOrAbilityWithSingleTarget`)
>   also legalizes targeting *abilities*, and `TargetFilter` has no single-target-count field
>   — either option is over-permissive (wrong cast legality). See EF-W-MISS-9.
> - **Ojutai, Soul of Winter** — authored + reviewed, then removed: the *targeted* variant of
>   `WheneverCreatureYouControlAttacks` silently drops its target (`enrich_spec_from_def`
>   hardcodes `targets: vec![]`), so the tap/detain resolves against nothing. Batch-2 review
>   HIGH. Engine gap, not authoring error — see EF-W-MISS-10; fix belongs to a PB/SR.
>
> Net authored Complete this wave: **33**.

### Per-group breakdown

| Group | Missing | Authorable | Blocked | Exists |
| --- | ---: | ---: | ---: | ---: |
| activated-sacrifice | 16 | 4 | 12 | 0 |
| activated-tap | 25 | 7 | 18 | 0 |
| attack-trigger | 28 | 3† | 25 | 0 |
| untap-phase | 12 | 5 | 7 | 0 |
| sacrifice-outlet | 5 | 1 | 4 | 0 |
| modal-choice | 37 | 4† | 33 | 0 |
| other | 23 | 3 | 18 | 2 |
| static-enchantment | 8 | 1 | 7 | 0 |
| token-create | 10 | 3 | 7 | 0 |
| body-only | 11 | 0 | 11 | 0 |
| discard-effect | 7 | 2 | 5 | 0 |
| draw | 6 | 1 | 5 | 0 |
| exile-play | 1 | 0 | 1 | 0 |
| removal-exile | 1 | 1 | 0 | 0 |
| **Total** | **194** | **33** | **159** | **2** |

† attack-trigger and modal-choice each lost one to a mid-wave demotion (Ojutai → EF-W-MISS-10;
Misdirection → EF-W-MISS-9). The **new-authoring** total shipped Complete this wave is **33**.
Exact list below (Ojutai and Misdirection struck through in the batch tables).

## False-missing (already on disk — do NOT author)

The report lists these as missing due to an apostrophe name-normalization quirk in
`authoring-report.py`'s plan-matching; both exist with correct `name:` fields and compile:
- **Steelshaper's Gift** → `steelshaper_s_gift.rs`
- **Dwynen's Elite** → `dwynen_s_elite.rs`

(Not a W-MISS deliverable to fix the report; noted for the reconciliation step so the
missing-count delta reads correctly.)

## Authorable now (35) — batched for authoring

### Batch 1 — tap-activated + sacrifice (12)
| Card | Reference def | Primitives / key clause |
| --- | --- | --- |
| Arbor Elf | `wirewood_lodge.rs` | `{T}` → `UntapPermanent` targeting `has_subtype "Forest"` (NOT a mana ability) |
| Contagion Clasp | `karns_bastion.rs`, `shambling_ghast.rs` | ETB `AddCounter{-1/-1}` on target; `{4},{T}` → `Effect::Proliferate` |
| Moggcatcher | tutor-to-battlefield defs | `{3},{T}` → `SearchLibrary{ has_subtype Goblin + **permanent card types** → Battlefield }` |
| Skyshroud Poacher | tutor-to-battlefield defs | same shape, subtype Elf, **constrain to permanent card types** |
| Sakura-Tribe Scout | `growth_spiral.rs` | `{T}` → `PutLandFromHandOntoBattlefield{tapped:false}` |
| Timberwatch Elf | `aspect_of_hydra.rs` | `{T}` → pump by `PermanentCount{subtype Elf, **EachPlayer**}` EOT |
| Wellwisher | `shamanic_revelation.rs` | `{T}` → `GainLife{PermanentCount{subtype Elf, **EachPlayer**}}` |
| Goblin Chirurgeon | `goblin_bombardment.rs`, `golgari_charm.rs` | `Cost::Sacrifice(has_subtype Goblin)` + `Regenerate{DeclaredTarget}`; do NOT set exclude_self |
| Goblin Lookout | `ezuri_renegade_leader.rs` | `Cost::Sequence([Tap, Sacrifice(Goblin)])` + team pump `AllCreaturesWithSubtype(Goblin)` EOT |
| Spore Frog | `fog.rs` | `Cost::SacrificeSelf` + `PreventAllCombatDamage` |
| Culling the Weak | `village_rites.rs` (cost) + `dark_ritual.rs` (effect) | `SpellAdditionalCost::SacrificeCreature` + `AddMana{black:4}` |
| Whirlpool Warrior | `winds_of_change.rs` | ETB shuffle-hand-into-library-draw-that-many (`WheelHand` Shuffle disposal); activated `Cost::Sequence([{R}, SacrificeSelf])` → `WheelHand{EachPlayer}` |

### Batch 2 — attack triggers, untap, wheels (12)
| Card | Reference def | Primitives / key clause |
| --- | --- | --- |
| Goblin Wardriver | Battle Cry (handled kw) | 2/2, single printed `KeywordAbility::BattleCry` |
| ~~Ojutai, Soul of Winter~~ **DEMOTED→blocked** | `sharktocrab.rs` | targeted attack trigger drops its target (EF-W-MISS-10); removed |
| Rhys the Exiled | dromoka/samut patterns | `WhenAttacks` → `GainLife{PermanentCount(Elf you control)}`; `{B},Sacrifice(Elf): Regenerate(Source)` |
| Triumphant Adventurer | `radha_heart_of_keld.rs`, `nadaar.rs` | Deathtouch kw; static `AddKeyword(FirstStrike)` on Source when `IsYourTurn`; `WhenAttacks` → `VentureIntoDungeon` |
| Aggravated Assault | `karlach_fury_of_avernus.rs` | Activated `{3}{R}{R}`, `SorcerySpeed`, `Sequence(UntapAll{creatures}, AdditionalCombatPhase{followed_by_main:true})` |
| Hyrax Tower Scout | `cloud_of_faeries.rs` | 3/3, ETB `UntapPermanent{DeclaredTarget}` |
| Mobilize | `seedborn_muse.rs` (UntapAll) | sorcery, `UntapAll{creatures you control}` |
| Vitalize | `seedborn_muse.rs` (UntapAll) | instant, `UntapAll{creatures you control}` |
| Wilderness Reclamation | `AtBeginningOfYourEndStep` + UntapAll | end-step trigger → `UntapAll{lands you control}` |
| Wheel of Fortune | `reforge_the_soul.rs` | `WheelHand{EachPlayer, Discard, Fixed(7)}` (no Miracle) |
| Tolarian Winds | `shattered_perception.rs` | `WheelHand{Controller, Discard, ThatMany}` |
| Fateful Showdown | `incendiary_command.rs`, `reforge_the_soul.rs` | `Sequence[DealDamage{TargetAny, CardCount{Hand}}, WheelHand{Controller, Discard, ThatMany}]` |

### Batch 3 — GY-land statics + misc (11)
| Card | Reference def | Primitives / key clause |
| --- | --- | --- |
| Crucible of Worlds | `perennial_behemoth.rs` | `StaticPlayFromGraveyard{LandsOnly}` |
| Ramunap Excavator | `perennial_behemoth.rs` | 2/3 vanilla creature + `StaticPlayFromGraveyard{LandsOnly}` |
| Icetill Explorer | `oracle_of_mul_daya.rs`, `perennial_behemoth.rs`, `retreat_to_kazandu.rs` | `AdditionalLandPlays{1}` + `StaticPlayFromGraveyard{LandsOnly}` + landfall → `MillCards{1}` |
| Bygone Colossus | `timeline_culler.rs` | 9/9 artifact creature + `AltCastAbility{Warp,{3},...}` + `Keyword(Warp)` |
| Omnath, Locus of the Roil | `avenger_of_zendikar.rs`, `omnath_locus_of_rage.rs` | ETB `DealDamage{TargetAny, PermanentCount{Elemental}}`; landfall → `Sequence[AddCounter, Conditional{YouControlNOrMore{8 lands} → DrawCards}]` (**verify combined targeted-landfall shape at review**) |
| Touch the Spirit Realm | `brutal_cathar.rs`, `boseiju_who_endures.rs` | ETB `ExileWithDelayedReturn{UpToN 1, WhenSourceLeaves}`; Channel = activated `Cost::Sequence[{1}{W}, DiscardSelf]` → exile at next end step (**verify Channel-from-hand + UpToN targeting at review**) |
| Ramunap.. see above | | |
| ~~Misdirection~~ **DEMOTED→blocked** | `force_of_will.rs`, `bolt_bend.rs` | single-target restriction inexpressible (EF-W-MISS-9); not authored |
| Flux Channeler | `venerated_rotpriest.rs`, `tezzerets_gambit.rs` | `WheneverYouCastSpell{noncreature_only:true}` → `Effect::Proliferate` |
| Dragonmaster Outcast | `simic_ascendancy.rs`, `murmuring_mystic.rs` | `AtBeginningOfYourUpkeep` + `intervening_if: YouControlNOrMore{6 lands}` → `CreateToken` (5/5 Dragon flying) |
| Revel in Riches | `hellkite_tyrant.rs`, `simic_ascendancy.rs` | `WheneverCreatureDies{controller: opponent}` → `CreateToken(Treasure)`; upkeep + `intervening_if: YouControlNOrMore{10 Treasure}` → `Effect::WinGame` |
| Scute Swarm | `avenger_of_zendikar.rs` | Landfall → `Conditional{YouControlNOrMore{6 lands} → CreateTokenCopy{Source}, else CreateToken(1/1 Insect)}` |

(The stray "Ramunap.. see above" row is a formatting duplicate — Ramunap Excavator is listed
once above; 11 distinct cards in Batch 3.)

## Multiplayer / timing traps for the authors (must-follow)

1. **Tribal counts span ALL players.** Timberwatch Elf / Wellwisher / Rhys count "Elves on
   the battlefield" or "Elves you control" — use `PermanentCount{controller: EachPlayer}` for
   the "on the battlefield" ones; `Controller` silently undercounts in multiplayer. (Rhys's
   lifegain is "Elves **you** control" → Controller; read the oracle per card.)
2. **Tutors must be constrained to permanent card types.** Moggcatcher/Skyshroud Poacher put a
   creature *onto the battlefield* — add `has_card_types` (permanent) alongside `has_subtype`,
   or a tribal instant/sorcery could illegally hit the battlefield.
3. **Sacrifice-a-Goblin costs must allow the source.** Goblin Chirurgeon/Goblin Lookout — do
   NOT set `exclude_self`; `eligible_sacrifice_targets` already includes the source.
4. **`UntapAll{creatures you control}`** for Mobilize/Vitalize/Aggravated Assault — these are
   "all creatures you control" (no exclude_self needed); fine. Copperhorn Scout ("each *other*")
   is BLOCKED precisely because `UntapAll` ignores `exclude_self`.
5. **Misdirection** — see the ⚠ caveat above; verify at review or demote.

## Blocked (157) — grouped by real current blocker

Every card below was traced clause-by-clause; each remains a genuine gap today. Not authored
(W6 policy: no TODO, no partial, no wrong game state).

### `EffectFilter::TriggeringCreature` missing (continuous buff on the attacker)
Atarka World Render (double strike EOT), Fervent Charge (+2/+2 EOT), Goblin Piledriver
(+2/+0 per attacking Goblin), Muxus Goblin Grandee (+1/+1 per other Goblin).

### "Defending player" target gap (the Hellrider gap — no target = the attacked player)
Brutal Hordechief, Raid Bombardment, Silumgar the Drifting Death (defending-player creature
filter), Norn's Decree, Cunning Rhetoric, Karazikar the Eye Tyrant.

### Granted keyword-trigger is a silent no-op (`AddKeyword` never synthesizes the trigger) — ✅ CLOSED (EF-W-MISS-3, PB-EF3b / scutemob-104, 2026-07-18)
FIXED: `layers::calculate_characteristics` now synthesizes the derived Melee/Battle Cry/Annihilator
trigger from post-layers keywords (shared `derived_attack_trigger_for_keyword` helper). **Adriana
Captain of the Guard** authored Complete (grant Melee — exercises the fix). **Skyhunter Strike Force**
authored partial: Flying + printed Melee modeled; the Lieutenant grant ("As long as you control your
commander …") is unrepresentable — no "control your commander" condition primitive → OOS-EF3b-1.
**Olivia Crimson Bride** remains out of scope (grants a *bespoke* trigger, not a keyword — not fixed
by this PB; still genuinely blocked).

### `MayPayOrElse` gated / optional-pay attack triggers
Frenzied Goblin, Hellkite Charger, Lightning Runner (+ energy), Formidable Speaker (ETB
optional discard-then-search).

### Card-invokable self-transform effect missing (all 11 body-only DFCs)
**RESOLVED (EF-W-MISS-6, scutemob-106, PB-EF5, 2026-07-18) — `Effect::TransformSelf` shipped;
Battle/Super Nova split out.** `Effect::TransformSelf` (unit variant, flips `ctx.source` in
place through the shared `transform_permanent_in_place` helper, CR 701.27f once-per-instruction
guarded) closes the self-transform gap. Cohort outcome (honest yield ~2, not ~7-9 — TransformSelf
is necessary for all 11 but sufficient for few): **docent_of_perfection** + **bloodline_keeper**
authored Complete; **delver_of_secrets** demoted Complete→partial (integrity — never transformed);
**thaumatic_compass** partial (front done; Spires back needs remove-from-combat → OOS-EF5-4g);
**growing_rites_of_itlimoc** partial (transform wired, ETB blocked). The rest carry distinct
out-of-scope 2nd blockers — see `ef-batch-plan-2026-07-17.md` §9 OOS-EF5-3 (edgar, fable,
nicol_bolas, grist — return-transformed as a NEW object, not in-place) and OOS-EF5-4
(legions_landing, westvale_abbey, etc.). **Invasion of Ikoria** (`CardType::Battle` — full CR 310
siege subsystem) → **OOS-EF5-1**; **Sephiroth** ("Super Nova" bespoke keyword action) → **OOS-EF5-2**;
both SPLIT OUT with justification (a bare enum variant without the machinery would ship wrong game
state — invariant #9). ~~The Effect enum has only `Meld`, no `Transform`/`TransformSelf`
(documented in `thaumatic_compass.rs`, `delver_of_secrets.rs`).~~

### Gated mana / chosen-color mana / mana restrictions
Lotus Petal, Jeweled Lotus, Orb of Dragonkind, Master of Dark Rites, Skirk Prospector,
Astral Cornucopia, Bloom Tender, Myr Convert, Wirewood Channeler, Heraldic Banner, Klauth,
Savage Ventmaw (persistent-mana flag), Carpet of Flowers, Gauntlet of Power, Utopia Sprawl,
Mana Geyser, High Tide, Treasure Nabber, Extraplanar Lens.

### Chosen-type: type-granting / per-counter scaling / ETB-counter / trigger-doubling
Arcane Adaptation, Door of Destinies, Metallic Mimic, Banner of Kinship, Coat of Arms,
Kindred Summons, Patriarch's Bidding, Mana Echoes, Roaming Throne (trigger-double),
Harmonic Prodigy (trigger-double), Realmwalker/Korlessa/Conspicuous Snoop/Dracogenesis
(no chosen/subtype play-from-top filter).

### Dynamic `max_cmc` / dynamic search filter / dynamic count
Birthing Ritual, Eldritch Evolution (max_cmc = N + sac MV), Genesis Wave, Nature's Rhythm,
Majestic Genesis, Rionya Fire Dancer, Hobgoblin Bandit Lord, Crackle with Power (up-to-X
targets).

### Sacrifice / cost / amount gaps
Momentous Fall (`ToughnessOfSacrificedCreature`), Victimize ("if you do" gate on
resolution-time sac), Chatterfang (variable-X sac), Heritage Druid / Earthcraft / Quirion
Ranger / Wirewood Symbiote (tap-a-creature / return-a-permanent activation cost), Lys Alana
Scarblade (filtered discard cost), Millikin (mill-as-cost), Insidious Dreams (variable-X
discard cost).

### Impulse / play-from-exile / cast-from-elsewhere-for-free
Dark-Dweller Oracle, Dauthi Voidwalker, Rod of Absorption, Conduit of Worlds, Fallen Shinobi,
Nashi, Silent-Blade Oni, Squee (self-cast from exile), Underworld Breach, Valakut Exploration,
Arcane Heist, As Foretold, Fist of Suns, Dracogenesis, Rishkar's Expertise, Winota.

### Library-dig / reveal-until / name-choice / reorder
Scroll Rack, Loot Exuberant Explorer, Kinnan, Eladamri Korvecdal, Lim-Dûl's Vault, Tainted
Pact, Demonic Consultation, Mitotic Manipulation, Kindred Summons, Gempalm Incinerator (no
`WhenYouCycle` trigger).

### Zone / GY-return / replacement / other
Life from the Loam (GY→hand return), Faith's Reward (this-turn provenance), Necropotence,
Memory Jar, Windfall (`WheelDraw` "greatest discarded"), Plaguecrafter (sac-else-discard
fallback), Kozilek (GY-only shuffle), Morality Shift (GY↔library exchange), New Blood
(text-change), Deepglow Skate (counter-doubling), Time Spiral (untap up-to-six lands),
Captain Howler (delayed trigger on a creature), Unshakable Tail, Ranger-Captain of Eos
(one-shot casting restriction), Contagion Engine (per-target-player scope), Stoneforge Mystic
(put nonland permanent from hand), The Chain Veil (loyalty re-activation), Magewright's Stone
(target "creature with a {T} ability"), Copperhorn Scout (`UntapAll` exclude_self),
Relentless Assault ("attacked this turn" filter), Intruder Alarm (global untap-prevention),
Clever Concealment / Ripples of Potential (phase-out), Ghyrson Starn (exactly-1-damage
trigger), Marvin/Leonin Shikari/Sigarda's Aid/Veilstone Amulet/Hand of the Praetors/
Ichormoon Gauntlet/Candlekeep Inspiration/Skrelv's Hive (Toxic)/Surly Badgersaur/Titania/
Delina/Hammers of Moradin/Auriok Steelshaper/Trench Behemoth/Raiyuu/Mystic Reflection/
Commandeer/Narset's Reversal/Bygone-adjacent/Sarkhan Soul Aflame/Opposition Agent/
An Offer You Can't Refuse (player-targeted token)/Augur of Autumn (Coven)/Caustic Bronco.

(The complete per-card blocker lines are preserved in the five triage-agent transcripts;
the groupings above name the precise missing primitive for each cluster.)

## Engine findings (filed as EF-W-MISS-*)

See `memory/card-authoring/w-miss-engine-findings-2026-07-17.md`. Most blockers here are
already-tracked gaps; the notable NEW finding is a latent legal-but-wrong bug in an already
**Complete** shipped def (`swan_song.rs` — token goes to the caster, not the countered spell's
controller), surfaced while triaging An Offer You Can't Refuse.

**EF-W-MISS-1 ✅ CLOSED (PB-EF2, `scutemob-102`, 2026-07-18).** `TokenSpec.recipient:
PlayerTarget` + `PlayerTarget::ControllerOfCounteredSpell` fix the recipient; `swan_song.rs`
is `Complete` again and **`an_offer_you_cant_refuse.rs` is now authored** (was the blocked
card named above under "player-targeted token").
</content>
</invoke>
