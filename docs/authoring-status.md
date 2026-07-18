<!-- DO NOT EDIT — regenerate via `python3 tools/authoring-report.py` -->

# Card Authoring Status — Canonical Report

**Generated:** 2026-07-18 01:10 UTC  
**Git:** `bcb82db2` on `feat/w-empty-author-the-110-authorable-empty-placeholder-card-def`  
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
| Card def files on disk | 1,748 | · |
| Authoring-plan target universe (snapshot 2026-03-10) | 1,636 | · |
| Plan cards with a def file (any-face match) | 1,442 | · |
| Plan cards still missing a def file | 194 | · |
| Bonus defs (on disk, outside plan) | 321 | · |
| Effective coverage vs plan target | **108%** (1,763 / 1,636) | — |
| Clean (no TODO/ENGINE-BLOCKED, non-empty abilities)  — 59.0% | 1,032 | +2 |
| With TODO markers | 659 | +1 |
| Empty `abilities: vec![]` placeholders | 57 | -3 |
| Total TODO lines across all defs | 983 | +1 |

## Authoring activity (git, by window)

| Window | New files added | Existing files modified |
| --- | ---: | ---: |
| last 7 days | 0 | 1,744 |
| last 30 days | 0 | 2,918 |
| last 90 days | 0 | 2,958 |
| last 1 year | 1,773 | 3,341 |

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
| `combat-keyword` | 187 / 187 | 100% | 82 | 97 | 8 |
| `draw` | 163 / 169 | 96% | 73 | 79 | 11 |
| `token-create` | 145 / 155 | 94% | 79 | 66 | 0 |
| `land-etb-tapped` | 138 / 138 | 100% | 116 | 22 | 0 |
| `other` | 108 / 131 | 82% | 67 | 41 | 0 |
| `modal-choice` | 68 / 105 | 65% | 32 | 36 | 0 |
| `mana-land` | 92 / 92 | 100% | 63 | 28 | 1 |
| `body-only` | 55 / 70 | 79% | 29 | 12 | 14 |
| `removal-destroy` | 56 / 56 | 100% | 33 | 21 | 2 |
| `counters-plus` | 49 / 49 | 100% | 24 | 24 | 1 |
| `land-fetch` | 45 / 45 | 100% | 27 | 17 | 1 |
| `attack-trigger` | 6 / 34 | 18% | 2 | 4 | 0 |
| `death-trigger` | 34 / 34 | 100% | 19 | 14 | 1 |
| `mana-artifact` | 34 / 34 | 100% | 14 | 18 | 2 |
| `activated-tap` | 2 / 27 | 7% | 1 | 1 | 0 |
| `pump-buff` | 27 / 27 | 100% | 17 | 10 | 0 |
| `cant-restriction` | 25 / 25 | 100% | 16 | 9 | 0 |
| `removal-damage-target` | 23 / 23 | 100% | 9 | 13 | 1 |
| `activated-sacrifice` | 3 / 19 | 16% | 1 | 2 | 0 |
| `mana-creature` | 19 / 19 | 100% | 12 | 7 | 0 |
| `graveyard-recursion` | 18 / 18 | 100% | 8 | 10 | 0 |
| `removal-damage-each` | 17 / 17 | 100% | 11 | 6 | 0 |
| `counter` | 16 / 16 | 100% | 8 | 5 | 3 |
| `removal-exile` | 13 / 14 | 93% | 4 | 5 | 4 |
| `untap-phase` | 1 / 13 | 8% | 0 | 0 | 1 |
| `cost-reduction` | 12 / 12 | 100% | 12 | 0 | 0 |
| `opponent-punish` | 12 / 12 | 100% | 4 | 8 | 0 |
| `equipment` | 11 / 11 | 100% | 6 | 5 | 0 |
| `tutor` | 11 / 11 | 100% | 9 | 1 | 1 |
| `removal-bounce` | 10 / 10 | 100% | 6 | 3 | 1 |
| `static-enchantment` | 0 / 8 | 0% | 0 | 0 | 0 |
| `discard-effect` | 0 / 7 | 0% | 0 | 0 | 0 |
| `scry-surveil` | 7 / 7 | 100% | 3 | 4 | 0 |
| `aura` | 6 / 6 | 100% | 3 | 2 | 1 |
| `etb-trigger` | 6 / 6 | 100% | 1 | 5 | 0 |
| `lifedrain` | 6 / 6 | 100% | 3 | 1 | 2 |
| `sacrifice-outlet` | 1 / 6 | 17% | 1 | 0 | 0 |
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

#### `discard-effect` — 0 / 7 (0%), authored split: 0 clean / 0 todo / 0 empty — **unwritten**

#### `static-enchantment` — 0 / 8 (0%), authored split: 0 clean / 0 todo / 0 empty — **unwritten**

#### `activated-tap` — 2 / 27 (7%), authored split: 1 clean / 1 todo / 0 empty — **unwritten**

| Card | Slug | Bucket |
| --- | --- | --- |
| Fauna Shaman | `fauna_shaman` | todo |
| Maze of Ith | `maze_of_ith` | clean |

#### `untap-phase` — 1 / 13 (8%), authored split: 0 clean / 0 todo / 1 empty — **engine-blocked**

| Card | Slug | Bucket |
| --- | --- | --- |
| Seedborn Muse | `seedborn_muse` | empty |

#### `activated-sacrifice` — 3 / 19 (16%), authored split: 1 clean / 2 todo / 0 empty — **engine-blocked**

| Card | Slug | Bucket |
| --- | --- | --- |
| Altar of Dementia | `altar_of_dementia` | clean |
| Birthing Pod | `birthing_pod` | todo |
| Bolas's Citadel | `bolass_citadel` | todo |

#### `sacrifice-outlet` — 1 / 6 (17%), authored split: 1 clean / 0 todo / 0 empty — **unwritten**

| Card | Slug | Bucket |
| --- | --- | --- |
| Life's Legacy | `lifes_legacy` | clean |

#### `attack-trigger` — 6 / 34 (18%), authored split: 2 clean / 4 todo / 0 empty — **engine-blocked**

| Card | Slug | Bucket |
| --- | --- | --- |
| Aurelia, the Warleader | `aurelia_the_warleader` | clean |
| Etali, Primal Storm | `etali_primal_storm` | todo |
| Hellrider | `hellrider` | todo |
| Sanctum Seeker | `sanctum_seeker` | clean |
| Shared Animosity | `shared_animosity` | todo |
| Six | `six` | todo |

## TODO classification (top 25)

Each TODO line is matched against engine-gap patterns. "OTHER" means unclassified — 
either a stale TODO (primitive now exists), a card-specific note, or a gap not yet 
in the classifier (`tools/authoring-report.py` `TODO_BUCKETS`). The OTHER bucket is 
the next thing to triage when the classifier table is grown.

| Gap bucket | TODO lines | Δ since last run |
| --- | ---: | ---: |
| OTHER (unclassified) | 599 | · |
| DSL gap (unspecified) | 123 | · |
| attack trigger (self / generic) | 26 | · |
| TriggerCondition::* missing variant | 17 | · |
| dynamic hexproof / protection | 17 | · |
| sacrifice as cost | 17 | +1 |
| Cost::* missing variant | 16 | · |
| replacement effect missing | 14 | · |
| EffectAmount::* missing variant | 12 | · |
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
| per-player effect dispatch | 4 | · |
| noncombat-damage prevent | 4 | · |
| ETB choice | 4 | · |
| impulse draw | 4 | · |
| CDA / dynamic P/T | 4 | · |

_…and 29 more buckets totaling 49 lines._

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
jeskas_will: // TODO: "choose both if commander" conditional entwine.
marisi_breaker_of_the_coil: // TODO: "Your opponents can't cast spells during combat" — phase-scoped CantCast not in DSL.
otharri_suns_glory: // TODO: "{2}{R}{W}, Tap an untapped Rebel you control: Return this card from your
roiling_dragonstorm: // TODO: "When a Dragon you control enters, return this to hand" —
song_of_freyalise: // TODO: Saga chapter I/II — grant `{T}: Add one mana of any color` to creatures you
teferi_temporal_archmage: // TODO: Emblem creation for "activate loyalty at instant speed" not in DSL.
tyvar_jubilant_brawler: // TODO: static — creatures you control can activate abilities as though they had haste
```

## ⚠ Completeness-marker drift

7 defs whose `completeness:` marker contradicts their comments. The marker is authoritative (it is what `validate_deck` reads), so fix whichever is stale.

- `ashnods_altar` — marked partial but has no TODO / ENGINE-BLOCKED comment
- `boggart_shenanigans` — marked partial but has no TODO / ENGINE-BLOCKED comment
- `olivia_voldaren` — marked partial but has no TODO / ENGINE-BLOCKED comment
- `phyrexian_tower` — marked partial but has no TODO / ENGINE-BLOCKED comment
- `scourge_of_valkas` — marked partial but has no TODO / ENGINE-BLOCKED comment
- `shaman_of_the_pack` — marked partial but has no TODO / ENGINE-BLOCKED comment
- `temple_of_the_dragon_queen` — marked partial but has no TODO / ENGINE-BLOCKED comment

## Recent card-touching commits

```
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
e2b1eb02 scutemob-88: /review fixes — unwrap 8 notes, 2 kind corrections, gate nit, EF-13
1de82c7c scutemob-88: marker sweep — audit all 742 non-Complete markers vs current engine
8ca8d2bc SR-10: migrate im 15.1 → imbl 7.0 (maintained fork)
b6f748f8 SR-6: extract card-defs and card-types crates — compile isolation for 1,749 defs
6c6d579e SR-2: mark 28 more known-wrong defs found by review (MEDIUM-1)
98a7a6a7 SR-2: registry gate for invariant #9 — completeness markers + duplicate CardId detection
b9397215 W6-cards: PB-AC9 backfill HIGH — Reforge the Soul stale Miracle marker
52a2b6f2 W6-prim: PB-AC9 — WheelHand + SetNoMaximumHandSize + token-doubling completeness
91885e98 W6-prim: PB-AC8 review fixes — 2 MEDIUM + 2 LOW closed
```

## Missing card-defs sidecar

The full list of 194 plan cards still missing on disk is at 
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
