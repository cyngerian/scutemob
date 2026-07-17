<!-- DO NOT EDIT — regenerate via `python3 tools/authoring-report.py` -->

# Card Authoring Status — Canonical Report

**Generated:** 2026-07-17 03:01 UTC  
**Git:** `a87b227b` on `feat/sr-33-88-dualtri-lands-are-complete-but-broken-effectchoose-`  
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
| Clean (no TODO/ENGINE-BLOCKED, non-empty abilities)  — 58.1% | 1,016 | -3 |
| With TODO markers | 670 | +3 |
| Empty `abilities: vec![]` placeholders | 62 | · |
| Total TODO lines across all defs | 1,045 | · |

## Authoring activity (git, by window)

| Window | New files added | Existing files modified |
| --- | ---: | ---: |
| last 7 days | 0 | 1,272 |
| last 30 days | 0 | 1,584 |
| last 90 days | 0 | 1,624 |
| last 1 year | 1,773 | 2,007 |

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
| `combat-keyword` | 187 / 187 | 100% | 77 | 102 | 8 |
| `draw` | 163 / 169 | 96% | 69 | 83 | 11 |
| `token-create` | 145 / 155 | 94% | 68 | 77 | 0 |
| `land-etb-tapped` | 138 / 138 | 100% | 118 | 20 | 0 |
| `other` | 108 / 131 | 82% | 67 | 41 | 0 |
| `modal-choice` | 68 / 105 | 65% | 31 | 37 | 0 |
| `mana-land` | 92 / 92 | 100% | 63 | 28 | 1 |
| `body-only` | 55 / 70 | 79% | 26 | 10 | 19 |
| `removal-destroy` | 56 / 56 | 100% | 34 | 20 | 2 |
| `counters-plus` | 49 / 49 | 100% | 21 | 27 | 1 |
| `land-fetch` | 45 / 45 | 100% | 27 | 17 | 1 |
| `attack-trigger` | 6 / 34 | 18% | 2 | 4 | 0 |
| `death-trigger` | 34 / 34 | 100% | 16 | 17 | 1 |
| `mana-artifact` | 34 / 34 | 100% | 22 | 10 | 2 |
| `activated-tap` | 2 / 27 | 7% | 0 | 2 | 0 |
| `pump-buff` | 27 / 27 | 100% | 16 | 11 | 0 |
| `cant-restriction` | 25 / 25 | 100% | 16 | 9 | 0 |
| `removal-damage-target` | 23 / 23 | 100% | 8 | 13 | 2 |
| `activated-sacrifice` | 3 / 19 | 16% | 1 | 2 | 0 |
| `mana-creature` | 19 / 19 | 100% | 14 | 5 | 0 |
| `graveyard-recursion` | 18 / 18 | 100% | 7 | 11 | 0 |
| `removal-damage-each` | 17 / 17 | 100% | 11 | 6 | 0 |
| `counter` | 16 / 16 | 100% | 6 | 6 | 4 |
| `removal-exile` | 13 / 14 | 93% | 5 | 4 | 4 |
| `untap-phase` | 1 / 13 | 8% | 0 | 0 | 1 |
| `cost-reduction` | 12 / 12 | 100% | 12 | 0 | 0 |
| `opponent-punish` | 12 / 12 | 100% | 3 | 9 | 0 |
| `equipment` | 11 / 11 | 100% | 6 | 5 | 0 |
| `tutor` | 11 / 11 | 100% | 8 | 2 | 1 |
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

#### `activated-tap` — 2 / 27 (7%), authored split: 0 clean / 2 todo / 0 empty — **engine-blocked**

| Card | Slug | Bucket |
| --- | --- | --- |
| Fauna Shaman | `fauna_shaman` | todo |
| Maze of Ith | `maze_of_ith` | todo |

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
| OTHER (unclassified) | 634 | · |
| DSL gap (unspecified) | 134 | · |
| attack trigger (self / generic) | 27 | · |
| TriggerCondition::* missing variant | 17 | · |
| dynamic hexproof / protection | 17 | · |
| Cost::* missing variant | 17 | · |
| sacrifice as cost | 16 | · |
| replacement effect missing | 14 | · |
| EffectAmount::* missing variant | 13 | · |
| TargetFilter missing field | 12 | · |
| combat-damage-to-player trigger | 10 | · |
| interactive / hidden-info choice | 10 | · |
| can't / must block-attack | 7 | · |
| can't be countered | 7 | · |
| opponent-action trigger | 7 | · |
| per-player effect dispatch | 6 | · |
| per-opponent upkeep | 6 | · |
| devotion | 5 | · |
| conditional static / grant | 5 | · |
| delayed triggers | 5 | · |
| equipment grants ability | 5 | · |
| X-scaled tokens | 4 | · |
| count-threshold static | 4 | · |
| untap-all / untap trigger | 4 | · |
| noncombat-damage prevent | 4 | · |

_…and 30 more buckets totaling 55 lines._

### Raw OTHER samples (read these to design new classifier buckets)

Showing 12 of 634 
unclassified TODO lines. If two or three of these have a common theme, that's a 
new bucket to add to `TODO_BUCKETS` in `tools/authoring-report.py`. Sample is 
deterministic (sorted by slug).

```
abstergo_entertainment: // TODO: {3}, {T}, Exile Abstergo Entertainment: Return up to one target historic card
bloodchief_ascension: // TODO: Both abilities are complex — end-step conditional counter placement needs
deadly_tempest: // TODO: The "each player loses life equal to creatures they controlled" requires
esper_sentinel: // TODO: Opponent-cast trigger with noncreature filter, once-per-turn,
go_for_the_throat: // TODO: "nonartifact creature" — no exclude_card_types on TargetFilter.
jeskas_will: // TODO: Mode 2 needs impulse-draw (exile top 3, play this turn).
marisi_breaker_of_the_coil: // TODO: "Your opponents can't cast spells during combat" — phase-scoped CantCast not in DSL.
out_of_the_tombs: // TODO: Upkeep counter + mill scaling with counter count not expressible.
ruthless_technomancer: // ENGINE-BLOCKED (activated ability): "{2}{B}, Sacrifice X artifacts: Return target
smugglers_surprise: // TODO: Spree mode 2 — put up to two creature cards from hand onto battlefield.
teferi_master_of_time: // TODO: Effect::PhaseOut not yet implemented. Placeholder preserves oracle index order.
tyvar_jubilant_brawler: // TODO: static — creatures you control can activate abilities as though they had haste
```

## Recent card-touching commits

```
e2b1eb02 scutemob-88: /review fixes — unwrap 8 notes, 2 kind corrections, gate nit, EF-13
1de82c7c scutemob-88: marker sweep — audit all 742 non-Complete markers vs current engine
8ca8d2bc SR-10: migrate im 15.1 → imbl 7.0 (maintained fork)
b6f748f8 SR-6: extract card-defs and card-types crates — compile isolation for 1,749 defs
6c6d579e SR-2: mark 28 more known-wrong defs found by review (MEDIUM-1)
98a7a6a7 SR-2: registry gate for invariant #9 — completeness markers + duplicate CardId detection
b9397215 W6-cards: PB-AC9 backfill HIGH — Reforge the Soul stale Miracle marker
52a2b6f2 W6-prim: PB-AC9 — WheelHand + SetNoMaximumHandSize + token-doubling completeness
91885e98 W6-prim: PB-AC8 review fixes — 2 MEDIUM + 2 LOW closed
12981e69 W6-cards: PB-AC8 backfill + card-review fixes
79135e64 W6-prim: PB-AC7 — /review LOW doc nits (SetCardTypes doc drift, CR 205.1b->205.1a)
cbcc02d8 W6-cards: PB-AC7 backfill — 5 clean cards + 2 partial-clause improvements
1caa8cc1 W6-prim: PB-AC7 engine — SetCreatureTypes/SetCardTypes + spell_subtype_filter
56602d5b W6-cards: PB-AC6 — /review polish (2 non-blocking observations)
bcc96cdf W6-cards: PB-AC6 — fix card review findings (2 HIGH, 1 MEDIUM)
e71b11d4 W6-cards: PB-AC6 backfill — partials, Kaito +1, and marker correction sweep
286c2b18 W6-cards: PB-AC6 backfill — 6 clean cards + integration tests
5920a5eb W6-prim: PB-AC5 review fixes — 2 HIGH hash-corruption defects
32bd607d W6-prim: PB-AC5 — Warp, Transmute, Exert, pitch alt-costs
5df8ed1a W6-cards: PB-AC4 — precise rakdos_charm ENGINE-BLOCKED marker
e9025936 W6-cards: PB-AC4 — fix 2 HIGH card review findings (golgari regenerate, abzan target filter)
6947ec36 W6-cards: PB-AC4 backfill — migrate modal cards to per-mode targeting
81fec080 W6-prim: PB-AC4 — per-mode target requirements on ModeSelection (CR 700.2c/700.2f)
d771b795 W6-prim: PB-AC3 card-review fixes — 4 HIGH wrong-game-state resolutions
0d274517 W6-prim: PB-AC3 review fix — Mirror Entity AddAllCreatureTypes to Layer 4 (TypeChange)
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
