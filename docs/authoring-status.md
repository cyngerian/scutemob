<!-- DO NOT EDIT — regenerate via `python3 tools/authoring-report.py` -->

# Card Authoring Status — Canonical Report

**Generated:** 2026-07-19 08:54 UTC  
**Git:** `38246a6e` on `main`  
**Source:** `tools/authoring-report.py`

This document is the single source of truth for card authoring progress. 
It is fully derived from the filesystem, the authoring plan JSON, and `git log`. 
Discussions of authoring strategy should reference this report, not stale prose docs.

**See [`authoring-status-guide.md`](authoring-status-guide.md) for how to read this report 
and what is intentionally NOT in it.**

---

## Headline

| Metric | Count | Δ since last run |
| --- | ---: | ---: |
| Card def files on disk | 1,803 | · |
| Authoring-plan target universe (snapshot 2026-03-10) | 1,636 | · |
| Plan cards with a def file (any-face match) | 1,501 | · |
| Plan cards still missing a def file | 135 | · |
| Bonus defs (on disk, outside plan) | 321 | · |
| Effective coverage vs plan target | **111%** (1,822 / 1,636) | — |
| Clean (no TODO/ENGINE-BLOCKED, non-empty abilities)  — 62.5% | 1,127 | +2 |
| With TODO markers | 524 | -1 |
| Empty `abilities: vec![]` placeholders | 152 | -1 |
| Total TODO lines across all defs | 956 | -2 |

## Authoring activity (git, by window)

| Window | New files added | Existing files modified |
| --- | ---: | ---: |
| last 7 days | 56 | 1,767 |
| last 30 days | 56 | 2,941 |
| last 90 days | 56 | 2,977 |
| last 1 year | 1,829 | 3,364 |

## Bonus defs outside the plan

The plan was a one-shot snapshot at 2026-03-10; 
any card authored before plan generation OR added since (without re-running the planner) 
appears here. These are real cards, not noise — typically EDH staples, ability-batch 
reference cards, or sample cards shipped alongside primitive batches.

| Source (commit prefix) | Count |
| --- | ---: |
| `W2` | 119 |
| `W1-B* (ability batches)` | 90 |
| `W6-cards` | 45 |
| `W5-cards` | 36 |
| `W6-prim` | 17 |
| `chore` | 11 |
| `W1-Morph` | 3 |

**By month added:** 2026-02: 137, 2026-03: 173, 2026-04: 11

## Coverage by authoring-plan group

"Clean" / "TODO" / "Empty" subdivide the *authored* count by file quality. 
Groups with high authored-but-not-clean ratios are TODO-debt — the cards exist but 
are blocked on engine primitives.

| Group | Auth / Total | % | Clean | TODO | Empty |
| --- | ---: | ---: | ---: | ---: | ---: |
| `combat-keyword` | 187 / 187 | 100% | 88 | 84 | 15 |
| `draw` | 164 / 169 | 97% | 78 | 69 | 17 |
| `token-create` | 148 / 155 | 95% | 82 | 50 | 16 |
| `land-etb-tapped` | 138 / 138 | 100% | 116 | 22 | 0 |
| `other` | 108 / 131 | 82% | 70 | 31 | 7 |
| `modal-choice` | 73 / 105 | 70% | 37 | 24 | 12 |
| `mana-land` | 92 / 92 | 100% | 64 | 27 | 1 |
| `body-only` | 64 / 70 | 91% | 38 | 10 | 16 |
| `removal-destroy` | 56 / 56 | 100% | 35 | 17 | 4 |
| `counters-plus` | 49 / 49 | 100% | 25 | 19 | 5 |
| `land-fetch` | 45 / 45 | 100% | 27 | 14 | 4 |
| `attack-trigger` | 19 / 34 | 56% | 15 | 3 | 1 |
| `death-trigger` | 34 / 34 | 100% | 20 | 9 | 5 |
| `mana-artifact` | 34 / 34 | 100% | 22 | 10 | 2 |
| `activated-tap` | 9 / 27 | 33% | 8 | 0 | 1 |
| `pump-buff` | 27 / 27 | 100% | 17 | 7 | 3 |
| `cant-restriction` | 25 / 25 | 100% | 16 | 5 | 4 |
| `removal-damage-target` | 23 / 23 | 100% | 10 | 11 | 2 |
| `activated-sacrifice` | 8 / 19 | 42% | 6 | 1 | 1 |
| `mana-creature` | 19 / 19 | 100% | 14 | 5 | 0 |
| `graveyard-recursion` | 18 / 18 | 100% | 8 | 6 | 4 |
| `removal-damage-each` | 17 / 17 | 100% | 11 | 5 | 1 |
| `counter` | 16 / 16 | 100% | 8 | 5 | 3 |
| `removal-exile` | 14 / 14 | 100% | 5 | 5 | 4 |
| `untap-phase` | 6 / 13 | 46% | 5 | 0 | 1 |
| `cost-reduction` | 12 / 12 | 100% | 12 | 0 | 0 |
| `opponent-punish` | 12 / 12 | 100% | 5 | 2 | 5 |
| `equipment` | 11 / 11 | 100% | 6 | 5 | 0 |
| `tutor` | 11 / 11 | 100% | 9 | 1 | 1 |
| `removal-bounce` | 10 / 10 | 100% | 6 | 3 | 1 |
| `static-enchantment` | 1 / 8 | 12% | 1 | 0 | 0 |
| `discard-effect` | 4 / 7 | 57% | 4 | 0 | 0 |
| `scry-surveil` | 7 / 7 | 100% | 3 | 4 | 0 |
| `aura` | 6 / 6 | 100% | 3 | 2 | 1 |
| `etb-trigger` | 6 / 6 | 100% | 1 | 4 | 1 |
| `lifedrain` | 6 / 6 | 100% | 3 | 1 | 2 |
| `sacrifice-outlet` | 6 / 6 | 100% | 6 | 0 | 0 |
| `lifegain` | 5 / 5 | 100% | 3 | 0 | 2 |
| `mana-other` | 5 / 5 | 100% | 3 | 2 | 0 |
| `removal-minus` | 4 / 4 | 100% | 2 | 1 | 1 |
| `exile-play` | 0 / 1 | 0% | 0 | 0 | 0 |
| `protection` | 1 / 1 | 100% | 0 | 1 | 0 |
| `x-spell` | 1 / 1 | 100% | 1 | 0 | 0 |

### Lagging groups (≥5 cards in plan, <50% authored)

For each lagging group, the table below lists the cards that ARE authored 
with their quality bucket. If most are `todo` or `empty`, the group is 
**engine-blocked** (cards exist but need primitives). If they are `clean`, 
the group is just **unwritten** (need authoring effort). This split tells 
you which kind of next-step work would unblock the group.

#### `static-enchantment` — 1 / 8 (12%), authored split: 1 clean / 0 todo / 0 empty — **unwritten**

| Card | Slug | Bucket |
| --- | --- | --- |
| Flux Channeler | `flux_channeler` | clean |

#### `activated-tap` — 9 / 27 (33%), authored split: 8 clean / 0 todo / 1 empty — **unwritten**

| Card | Slug | Bucket |
| --- | --- | --- |
| Arbor Elf | `arbor_elf` | clean |
| Contagion Clasp | `contagion_clasp` | clean |
| Fauna Shaman | `fauna_shaman` | empty |
| Maze of Ith | `maze_of_ith` | clean |
| Moggcatcher | `moggcatcher` | clean |
| Sakura-Tribe Scout | `sakura_tribe_scout` | clean |
| Skyshroud Poacher | `skyshroud_poacher` | clean |
| Timberwatch Elf | `timberwatch_elf` | clean |
| Wellwisher | `wellwisher` | clean |

#### `activated-sacrifice` — 8 / 19 (42%), authored split: 6 clean / 1 todo / 1 empty — **unwritten**

| Card | Slug | Bucket |
| --- | --- | --- |
| Altar of Dementia | `altar_of_dementia` | clean |
| An Offer You Can't Refuse | `an_offer_you_cant_refuse` | clean |
| Birthing Pod | `birthing_pod` | empty |
| Bolas's Citadel | `bolass_citadel` | todo |
| Goblin Chirurgeon | `goblin_chirurgeon` | clean |
| Goblin Lookout | `goblin_lookout` | clean |
| Spore Frog | `spore_frog` | clean |
| Whirlpool Warrior | `whirlpool_warrior` | clean |

#### `untap-phase` — 6 / 13 (46%), authored split: 5 clean / 0 todo / 1 empty — **unwritten**

| Card | Slug | Bucket |
| --- | --- | --- |
| Aggravated Assault | `aggravated_assault` | clean |
| Hyrax Tower Scout | `hyrax_tower_scout` | clean |
| Mobilize | `mobilize` | clean |
| Seedborn Muse | `seedborn_muse` | empty |
| Vitalize | `vitalize` | clean |
| Wilderness Reclamation | `wilderness_reclamation` | clean |

## TODO classification (top 25)

Each TODO line is matched against engine-gap patterns. "OTHER" means unclassified — 
either a stale TODO (primitive now exists), a card-specific note, or a gap not yet 
in the classifier (`tools/authoring-report.py` `TODO_BUCKETS`). The OTHER bucket is 
the next thing to triage when the classifier table is grown.

| Gap bucket | TODO lines | Δ since last run |
| --- | ---: | ---: |
| OTHER (unclassified) | 586 | -1 |
| DSL gap (unspecified) | 120 | · |
| attack trigger (self / generic) | 26 | · |
| TriggerCondition::* missing variant | 17 | · |
| dynamic hexproof / protection | 17 | · |
| replacement effect missing | 14 | · |
| Cost::* missing variant | 13 | · |
| EffectAmount::* missing variant | 12 | · |
| sacrifice as cost | 11 | · |
| combat-damage-to-player trigger | 10 | · |
| interactive / hidden-info choice | 10 | -1 |
| can't / must block-attack | 7 | · |
| can't be countered | 7 | · |
| opponent-action trigger | 7 | · |
| TargetFilter missing field | 7 | · |
| per-opponent upkeep | 6 | · |
| conditional static / grant | 5 | · |
| delayed triggers | 5 | · |
| equipment grants ability | 5 | · |
| untap-all / untap trigger | 4 | · |
| noncombat-damage prevent | 4 | · |
| ETB choice | 4 | · |
| impulse draw | 4 | · |
| CDA / dynamic P/T | 4 | · |
| devotion | 4 | · |

_…and 27 more buckets totaling 47 lines._

### Raw OTHER samples (read these to design new classifier buckets)

Showing 12 of 586 
unclassified TODO lines. If two or three of these have a common theme, that's a 
new bucket to add to `TODO_BUCKETS` in `tools/authoring-report.py`. Sample is 
deterministic (sorted by slug).

```
abstergo_entertainment: // TODO: {3}, {T}, Exile Abstergo Entertainment: Return up to one target historic card
blood_tribute: // TODO: "if kicked, gain life equal to life lost" needs conditional.
deadly_tempest: // TODO: The "each player loses life equal to creatures they controlled" requires
exuberant_fuseling: // TODO: "whenever another creature or artifact you control is put into a graveyard
goblin_king: // TODO: AllCreaturesWithSubtype includes Goblin King itself — "other" semantics
jeskas_will: // TODO: "choose both if commander" conditional entwine.
marionette_apprentice: // ENGINE-BLOCKED: "Whenever another creature or artifact you control is put into
out_of_the_tombs: // TODO: Upkeep counter + mill scaling with counter count not expressible.
ruthless_winnower: // TODO: "non-Elf creature" filter — SacrificePermanents has no subtype exclusion.
song_of_freyalise: // TODO: Saga chapter III — +1/+1 counters on all creatures you control plus vigilance,
teferi_temporal_archmage: // TODO: Emblem creation for "activate loyalty at instant speed" not in DSL.
tyvar_jubilant_brawler: // TODO: static — creatures you control can activate abilities as though they had haste
```

## ⚠ Completeness-marker drift

4 defs whose `completeness:` marker contradicts their comments. The marker is authoritative (it is what `validate_deck` reads), so fix whichever is stale.

- `ashnods_altar` — marked partial but has no TODO / ENGINE-BLOCKED comment
- `boggart_shenanigans` — marked partial but has no TODO / ENGINE-BLOCKED comment
- `phyrexian_tower` — marked partial but has no TODO / ENGINE-BLOCKED comment
- `temple_of_the_dragon_queen` — marked partial but has no TODO / ENGINE-BLOCKED comment

## Recent card-touching commits

```
63148132 W6-prim: PB-OS8 — Effect::LookAtTopThenPlace + TargetFilter.min_cmc_amount (OOS-EF10-1 + OS6-deferred-(d))
2beaba4f W6-prim: PB-OS7 — author silumgar_the_drifting_death.rs (Complete)
bd15b45b W6-prim: PB-OS6 review LOW — soften delver 'strictly beneficial' comment wording
969ef404 W6-prim: PB-OS6 wire bump (PROTOCOL 20->21, HASH 57->58) + card defs
8c31c1fd W6-prim: PB-OS5 (OOS-EF4-1) — dynamic relative-count EffectAmount
55664ad8 scutemob-134: PB-OS4b card-def message fixes + wip checklist
be9f371c W6-prim: PB-OS4 /review nit — Fable back-face comment names OOS-OS4-2 non-functional status
7945c975 W6-prim: PB-OS4 fix pass — fmt gate (scutemob-130)
f5a44ab6 W6-prim: PB-OS4 fix pass — ship narrowed (scutemob-130)
fc3ae4ef W6-prim: PB-OS4 card defs + tests — Fable + Edgar return-transformed
e16cd0c8 W6-prim: PB-OS3 — WhenTappedForMana trigger target dispatch (OOS-EF6-1)
95c8a632 scutemob-128: PB-OS2 — thread sacrificed-creature LKI through the optional-cost path (EF-EF1-A)
a8eb45b5 scutemob-114: PB-EF12 — granted any_color ManaAbility color choice (EF-W-PB2-3) — CLOSES THE EF QUEUE
50a83faf scutemob-112: PB-EF11 COMMIT 2 — spell-only TargetSpellWithSingleTarget + Misdirection (PROTOCOL 17, HASH 55)
135ef9e6 scutemob-112: PB-EF11 COMMIT 1 — WheelDraw::GreatestDiscarded + Windfall (PROTOCOL 16, HASH 54)
9418011b scutemob-111: PB-EF10 COMMIT 3 — Condition::SacrificeFired + version bump (PROTOCOL 15, HASH 53)
051887f2 scutemob-111: PB-EF10 COMMIT 2 — runtime search cap (max_cmc_amount + ManaValueOfSacrificedCreature)
ad9755ff scutemob-111: PB-EF10 COMMIT 1 — SacrificedCreatureLki data-model migration + ToughnessOfSacrificedCreature
eba28604 scutemob-110: PB-EF9 — EffectDuration::WhileYouControlSource (EF-W-PB2-5)
a7ae66ce scutemob-109: PB-EF8 /review LOW — elvish_spirit_guide oracle_text card→creature
3a5f1678 scutemob-109: PB-EF8 — Cost::ExileSelfFromHand (activation from hand)
1574aa17 scutemob-108: PB-EF7 review fixes — LKI discriminator + validation-branch tests
bd43762b scutemob-108: PB-EF7 card fixes — Goblin Cratermaker + Cankerbloom to Complete
a4319e8d scutemob-108: PB-EF7 corpus-wide modes: None, on Activated ability defs (mechanical)
7f6d5082 scutemob-107: PB-EF6 card-def fixups — vengeful_bloodwitch comment, forbidden_orchard revert
```

## Missing card-defs sidecar

The full list of 135 plan cards still missing on disk is at 
`docs/authoring-status-missing.txt` (tab-separated `group<TAB>name`, sorted by group). 
Use it as a batch-author worklist.

---

## How to update this report

```
python3 tools/authoring-report.py
```

To extend the TODO classifier, add `(re.compile(...), "bucket name")` tuples to 
`TODO_BUCKETS` in `tools/authoring-report.py` and re-run.

To change the universe target or plan source, edit `PLAN` at the top of the script.
