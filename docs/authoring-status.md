<!-- DO NOT EDIT — regenerate via `python3 tools/authoring-report.py` -->

# Card Authoring Status — Canonical Report

**Generated:** 2026-07-18 09:46 UTC  
**Git:** `22acde6b` on `feat/pb-ef3b-granted-keyword-triggers-fire-meleebattle-cryannihil`  
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
| Card def files on disk | 1,787 | +2 |
| Authoring-plan target universe (snapshot 2026-03-10) | 1,636 | · |
| Plan cards with a def file (any-face match) | 1,481 | +2 |
| Plan cards still missing a def file | 155 | -2 |
| Bonus defs (on disk, outside plan) | 321 | · |
| Effective coverage vs plan target | **110%** (1,802 / 1,636) | — |
| Clean (no TODO/ENGINE-BLOCKED, non-empty abilities)  — 60.2% | 1,076 | +1 |
| With TODO markers | 553 | +1 |
| Empty `abilities: vec![]` placeholders | 158 | · |
| Total TODO lines across all defs | 973 | +1 |

## Authoring activity (git, by window)

| Window | New files added | Existing files modified |
| --- | ---: | ---: |
| last 7 days | 39 | 1,744 |
| last 30 days | 39 | 2,918 |
| last 90 days | 39 | 2,958 |
| last 1 year | 1,812 | 3,341 |

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
| `combat-keyword` | 187 / 187 | 100% | 83 | 87 | 17 |
| `draw` | 164 / 169 | 97% | 78 | 69 | 17 |
| `token-create` | 148 / 155 | 95% | 82 | 50 | 16 |
| `land-etb-tapped` | 138 / 138 | 100% | 116 | 22 | 0 |
| `other` | 108 / 131 | 82% | 67 | 33 | 8 |
| `modal-choice` | 72 / 105 | 69% | 36 | 24 | 12 |
| `mana-land` | 92 / 92 | 100% | 63 | 28 | 1 |
| `body-only` | 55 / 70 | 79% | 29 | 10 | 16 |
| `removal-destroy` | 56 / 56 | 100% | 33 | 19 | 4 |
| `counters-plus` | 49 / 49 | 100% | 24 | 20 | 5 |
| `land-fetch` | 45 / 45 | 100% | 27 | 14 | 4 |
| `attack-trigger` | 14 / 34 | 41% | 10 | 2 | 2 |
| `death-trigger` | 34 / 34 | 100% | 19 | 10 | 5 |
| `mana-artifact` | 34 / 34 | 100% | 14 | 18 | 2 |
| `activated-tap` | 9 / 27 | 33% | 8 | 0 | 1 |
| `pump-buff` | 27 / 27 | 100% | 17 | 7 | 3 |
| `cant-restriction` | 25 / 25 | 100% | 16 | 5 | 4 |
| `removal-damage-target` | 23 / 23 | 100% | 9 | 12 | 2 |
| `activated-sacrifice` | 8 / 19 | 42% | 6 | 1 | 1 |
| `mana-creature` | 19 / 19 | 100% | 12 | 7 | 0 |
| `graveyard-recursion` | 18 / 18 | 100% | 8 | 6 | 4 |
| `removal-damage-each` | 17 / 17 | 100% | 11 | 5 | 1 |
| `counter` | 16 / 16 | 100% | 8 | 5 | 3 |
| `removal-exile` | 14 / 14 | 100% | 5 | 5 | 4 |
| `untap-phase` | 6 / 13 | 46% | 5 | 0 | 1 |
| `cost-reduction` | 12 / 12 | 100% | 12 | 0 | 0 |
| `opponent-punish` | 12 / 12 | 100% | 4 | 2 | 6 |
| `equipment` | 11 / 11 | 100% | 6 | 5 | 0 |
| `tutor` | 11 / 11 | 100% | 9 | 1 | 1 |
| `removal-bounce` | 10 / 10 | 100% | 6 | 3 | 1 |
| `static-enchantment` | 1 / 8 | 12% | 1 | 0 | 0 |
| `discard-effect` | 3 / 7 | 43% | 3 | 0 | 0 |
| `scry-surveil` | 7 / 7 | 100% | 3 | 4 | 0 |
| `aura` | 6 / 6 | 100% | 3 | 2 | 1 |
| `etb-trigger` | 6 / 6 | 100% | 1 | 4 | 1 |
| `lifedrain` | 6 / 6 | 100% | 3 | 1 | 2 |
| `sacrifice-outlet` | 2 / 6 | 33% | 2 | 0 | 0 |
| `lifegain` | 5 / 5 | 100% | 3 | 0 | 2 |
| `mana-other` | 5 / 5 | 100% | 2 | 3 | 0 |
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

#### `sacrifice-outlet` — 2 / 6 (33%), authored split: 2 clean / 0 todo / 0 empty — **unwritten**

| Card | Slug | Bucket |
| --- | --- | --- |
| Culling the Weak | `culling_the_weak` | clean |
| Life's Legacy | `lifes_legacy` | clean |

#### `attack-trigger` — 14 / 34 (41%), authored split: 10 clean / 2 todo / 2 empty — **unwritten**

| Card | Slug | Bucket |
| --- | --- | --- |
| Adriana, Captain of the Guard | `adriana_captain_of_the_guard` | clean |
| Aurelia, the Warleader | `aurelia_the_warleader` | clean |
| Copperhorn Scout | `copperhorn_scout` | clean |
| Etali, Primal Storm | `etali_primal_storm` | empty |
| Goblin Wardriver | `goblin_wardriver` | clean |
| Hellrider | `hellrider` | clean |
| Ojutai, Soul of Winter | `ojutai_soul_of_winter` | clean |
| Raid Bombardment | `raid_bombardment` | clean |
| Rhys the Exiled | `rhys_the_exiled` | clean |
| Sanctum Seeker | `sanctum_seeker` | clean |
| Shared Animosity | `shared_animosity` | empty |
| Six | `six` | todo |
| Skyhunter Strike Force | `skyhunter_strike_force` | todo |
| Triumphant Adventurer | `triumphant_adventurer` | clean |

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

#### `discard-effect` — 3 / 7 (43%), authored split: 3 clean / 0 todo / 0 empty — **unwritten**

| Card | Slug | Bucket |
| --- | --- | --- |
| Fateful Showdown | `fateful_showdown` | clean |
| Tolarian Winds | `tolarian_winds` | clean |
| Wheel of Fortune | `wheel_of_fortune` | clean |

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
| OTHER (unclassified) | 599 | +1 |
| DSL gap (unspecified) | 122 | · |
| attack trigger (self / generic) | 26 | · |
| TriggerCondition::* missing variant | 17 | · |
| dynamic hexproof / protection | 17 | · |
| Cost::* missing variant | 16 | · |
| replacement effect missing | 14 | · |
| EffectAmount::* missing variant | 12 | · |
| sacrifice as cost | 11 | · |
| combat-damage-to-player trigger | 10 | · |
| interactive / hidden-info choice | 10 | · |
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

_…and 28 more buckets totaling 46 lines._

### Raw OTHER samples (read these to design new classifier buckets)

Showing 12 of 599 
unclassified TODO lines. If two or three of these have a common theme, that's a 
new bucket to add to `TODO_BUCKETS` in `tools/authoring-report.py`. Sample is 
deterministic (sorted by slug).

```
abstergo_entertainment: // TODO: {3}, {T}, Exile Abstergo Entertainment: Return up to one target historic card
blood_tribute: // TODO: Kicker cost "tap a Vampire" is non-mana kicker.
deadly_tempest: // TODO: The "each player loses life equal to creatures they controlled" requires
experimental_augury: // TODO: Interactive "choose 1 of 3" — M10 player choice. Approximated as
goblin_lackey: // TODO: "put a Goblin from hand onto battlefield" — needs MoveZone from
joraga_treespeaker: // TODO: Level up mechanic not in DSL — no LevelUp keyword or level-based ability gating.
marisi_breaker_of_the_coil: // TODO: "goad each creature that player controls" — ForEach over DamagedPlayer's creatures
out_of_the_tombs: // TODO: Upkeep counter + mill scaling with counter count not expressible.
ruthless_technomancer: // ENGINE-BLOCKED (activated ability): "{2}{B}, Sacrifice X artifacts: Return target
song_of_freyalise: // TODO: Saga chapter I/II — grant `{T}: Add one mana of any color` to creatures you
teferi_temporal_archmage: // TODO: Emblem creation for "activate loyalty at instant speed" not in DSL.
tyvar_jubilant_brawler: // TODO: static — creatures you control can activate abilities as though they had haste
```

## ⚠ Completeness-marker drift

7 defs whose `completeness:` marker contradicts their comments. The marker is authoritative (it is what `validate_deck` reads), so fix whichever is stale.

- `ashnods_altar` — marked partial but has no TODO / ENGINE-BLOCKED comment
- `boggart_shenanigans` — marked partial but has no TODO / ENGINE-BLOCKED comment
- `disciple_of_freyalise` — marked partial but has no TODO / ENGINE-BLOCKED comment
- `olivia_voldaren` — marked partial but has no TODO / ENGINE-BLOCKED comment
- `phyrexian_tower` — marked partial but has no TODO / ENGINE-BLOCKED comment
- `scourge_of_valkas` — marked partial but has no TODO / ENGINE-BLOCKED comment
- `temple_of_the_dragon_queen` — marked partial but has no TODO / ENGINE-BLOCKED comment

## Recent card-touching commits

```
43e73b32 scutemob-104: PB-EF3b — granted keyword-triggers fire (Melee/Battle Cry/Annihilator)
86aa9cca scutemob-103: PB-EF3 cards — Ojutai Soul of Winter, Hellrider, Raid Bombardment
6f2b299d scutemob-102: PB-EF2 — CreateToken player-scoped recipient (EF-W-MISS-1)
38fa59c3 scutemob-101: EF-13 (Option A) — reclassify 101 no-behaviour Partial defs to Inert
bfdda877 Merge branch 'main' into feat/pb-ef1-excludeself-enforcement-sweep-honor-the-field-at-ever
34ded5ee scutemob-100: demote swan_song Complete -> known_wrong (EF-W-MISS-1)
60e9eb00 scutemob-99: PB-EF1 cards + exclude_self regression tests
eb9d7e34 scutemob-97: W-MISS — author 33 missing-file cards Complete (coverage 59.0% -> 59.8%)
205282d3 scutemob-96: W-EMPTY — author authorable empty-placeholder defs
5321ebef scutemob-95: W-PB2 batch 5 — targeting/alt-cost/misc (8 Complete) + boggart demotion + test fixes
7ee2a68d scutemob-95: W-PB2 batch 4 — triggers (10 Complete)
e7e304b8 scutemob-95: W-PB2 batch 3 — dynamic P/T + static grants (9 Complete)
51d93961 scutemob-95: W-PB2 batch 2 — count-scaled amounts/tokens (10 Complete)
ba1fe756 scutemob-95: W-PB2 batch 1 — target-filter fixes (11 Complete, patriars_seal known_wrong)
2a1f0b60 SR-37: SF-11 + SF-12 — gate the any-color mana stubs; land gate sees them
530ba541 SR-36: close the review's 3 MEDIUMs — 0 HIGH found
91ce106c SR-36: reconcile markers — 3 upgrades, 2 stale notes, and a gate asymmetry SF-8 exposed
ae14ed5a scutemob-91: SR-35 — rustfmt gate over the card-def corpus (was checking zero files)
1fa03bc6 scutemob-90: apply SR-34 review fixes (0 HIGH, 5 MEDIUM, 3 LOW — all 8 resolved)
5cadf2ca scutemob-90: SR-34 roster reconciliation (criterion 4767) — 27 defs probed, 17 markers, 10 certified
b0290d2c scutemob-90: SR-34 §9 — un-demote the 3 horizon lands, extend the SR-33 colour gate to composite costs
247437f1 scutemob-90: SR-34 steps 1-6 — ManaAbility gains mana_cost/life_cost, handle_tap_for_mana pays them, mana-ability lowering widens from bare Cost::Tap
104bd5e3 scutemob-89: review fixes — gate AddManaChoice (3rd stub), correct 3 wrong findings, fmt
a25f87c8 scutemob-89: card review fixes — dimir_guildgate stale comment + oracle text; SF-6
5dcca855 scutemob-89: SR-33 — dual/tri lands produce every printed colour; gate the Choose stub
```

## Missing card-defs sidecar

The full list of 155 plan cards still missing on disk is at 
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
