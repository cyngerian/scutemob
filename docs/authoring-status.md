<!-- DO NOT EDIT — regenerate via `python3 tools/authoring-report.py` -->

# Card Authoring Status — Canonical Report

**Generated:** 2026-04-30 00:08 UTC  
**Git:** `ad793012` on `main`  
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
| Clean (no TODO, non-empty abilities)  — 52.3% | 915 | · |
| With TODO markers | 652 | · |
| Empty `abilities: vec![]` placeholders | 181 | · |
| Total TODO lines across all defs | 1,187 | · |

## Authoring activity (git, by window)

| Window | New files added | Existing files modified |
| --- | ---: | ---: |
| last 7 days | 0 | 26 |
| last 30 days | 278 | 331 |
| last 90 days | 1,773 | 1,147 |
| last 1 year | 1,773 | 1,147 |

## Bonus defs outside the plan

The plan was a one-shot snapshot at 2026-03-10; 
any card authored before plan generation OR added since (without re-running the planner) 
appears here. These are real cards, not noise — typically EDH staples, ability-batch 
reference cards, or sample cards shipped alongside primitive batches.

| Source (commit prefix) | Count |
| --- | ---: |
| `W2` | 112 |
| `W1-B* (ability batches)` | 90 |
| `W6-cards` | 47 |
| `W5-cards` | 41 |
| `W6-prim` | 17 |
| `chore` | 11 |
| `W1-Morph` | 3 |

**By month added:** 2026-02: 133, 2026-03: 177, 2026-04: 11

## Coverage by authoring-plan group

"Clean" / "TODO" / "Empty" subdivide the *authored* count by file quality. 
Groups with high authored-but-not-clean ratios are TODO-debt — the cards exist but 
are blocked on engine primitives.

| Group | Auth / Total | % | Clean | TODO | Empty |
| --- | ---: | ---: | ---: | ---: | ---: |
| `combat-keyword` | 187 / 187 | 100% | 75 | 102 | 10 |
| `draw` | 163 / 169 | 96% | 57 | 81 | 25 |
| `token-create` | 145 / 155 | 94% | 17 | 66 | 62 |
| `land-etb-tapped` | 138 / 138 | 100% | 116 | 20 | 2 |
| `other` | 108 / 131 | 82% | 61 | 47 | 0 |
| `modal-choice` | 68 / 105 | 65% | 25 | 43 | 0 |
| `mana-land` | 92 / 92 | 100% | 73 | 16 | 3 |
| `body-only` | 55 / 70 | 79% | 23 | 10 | 22 |
| `removal-destroy` | 56 / 56 | 100% | 33 | 16 | 7 |
| `counters-plus` | 49 / 49 | 100% | 17 | 32 | 0 |
| `land-fetch` | 45 / 45 | 100% | 25 | 19 | 1 |
| `attack-trigger` | 6 / 34 | 18% | 2 | 4 | 0 |
| `death-trigger` | 34 / 34 | 100% | 18 | 16 | 0 |
| `mana-artifact` | 34 / 34 | 100% | 21 | 10 | 3 |
| `activated-tap` | 2 / 27 | 7% | 1 | 1 | 0 |
| `pump-buff` | 27 / 27 | 100% | 16 | 11 | 0 |
| `cant-restriction` | 25 / 25 | 100% | 16 | 9 | 0 |
| `removal-damage-target` | 23 / 23 | 100% | 4 | 14 | 5 |
| `activated-sacrifice` | 3 / 19 | 16% | 1 | 2 | 0 |
| `mana-creature` | 19 / 19 | 100% | 14 | 5 | 0 |
| `graveyard-recursion` | 18 / 18 | 100% | 7 | 11 | 0 |
| `removal-damage-each` | 17 / 17 | 100% | 10 | 7 | 0 |
| `counter` | 16 / 16 | 100% | 4 | 6 | 6 |
| `removal-exile` | 13 / 14 | 93% | 5 | 2 | 6 |
| `untap-phase` | 1 / 13 | 8% | 0 | 1 | 0 |
| `cost-reduction` | 12 / 12 | 100% | 4 | 0 | 8 |
| `opponent-punish` | 12 / 12 | 100% | 2 | 10 | 0 |
| `equipment` | 11 / 11 | 100% | 6 | 5 | 0 |
| `tutor` | 11 / 11 | 100% | 6 | 3 | 2 |
| `removal-bounce` | 10 / 10 | 100% | 5 | 4 | 1 |
| `static-enchantment` | 0 / 8 | 0% | 0 | 0 | 0 |
| `discard-effect` | 0 / 7 | 0% | 0 | 0 | 0 |
| `scry-surveil` | 7 / 7 | 100% | 4 | 2 | 1 |
| `aura` | 6 / 6 | 100% | 2 | 3 | 1 |
| `etb-trigger` | 6 / 6 | 100% | 1 | 5 | 0 |
| `lifedrain` | 6 / 6 | 100% | 2 | 2 | 2 |
| `sacrifice-outlet` | 1 / 6 | 17% | 1 | 0 | 0 |
| `lifegain` | 5 / 5 | 100% | 3 | 0 | 2 |
| `mana-other` | 5 / 5 | 100% | 1 | 2 | 2 |
| `removal-minus` | 4 / 4 | 100% | 2 | 1 | 1 |
| `exile-play` | 0 / 1 | 0% | 0 | 0 | 0 |
| `protection` | 1 / 1 | 100% | 0 | 1 | 0 |
| `x-spell` | 1 / 1 | 100% | 0 | 1 | 0 |

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

#### `untap-phase` — 1 / 13 (8%), authored split: 0 clean / 1 todo / 0 empty — **engine-blocked**

| Card | Slug | Bucket |
| --- | --- | --- |
| Seedborn Muse | `seedborn_muse` | todo |

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
| OTHER (unclassified) | 682 | · |
| DSL gap (unspecified) | 149 | · |
| attack trigger (self / generic) | 31 | · |
| Cost::* missing variant | 22 | · |
| replacement effect missing | 21 | · |
| TriggerCondition::* missing variant | 19 | · |
| EffectAmount::* missing variant | 19 | · |
| dynamic hexproof / protection | 17 | · |
| sacrifice as cost | 17 | · |
| untap-all / untap trigger | 16 | · |
| TargetFilter missing field | 16 | · |
| CDA / dynamic P/T | 12 | · |
| interactive / hidden-info choice | 11 | · |
| combat-damage-to-player trigger | 10 | · |
| opponent-action trigger | 9 | · |
| X-scaled tokens | 8 | · |
| can't / must block-attack | 8 | · |
| can't be countered | 7 | · |
| no-maximum-hand-size | 7 | · |
| count-threshold static | 6 | · |
| proliferate trigger | 6 | · |
| per-opponent upkeep | 6 | · |
| devotion | 5 | · |
| conditional static / grant | 5 | · |
| counter-placed trigger | 5 | · |

_…and 33 more buckets totaling 73 lines._

### Raw OTHER samples (read these to design new classifier buckets)

Showing 12 of 682 
unclassified TODO lines. If two or three of these have a common theme, that's a 
new bucket to add to `TODO_BUCKETS` in `tools/authoring-report.py`. Sample is 
deterministic (sorted by slug).

```
abstergo_entertainment: // TODO: {3}, {T}, Exile Abstergo Entertainment: Return up to one target historic card
bonecrusher_giant: // TODO: "Damage can't be prevented this turn" — no prevention-removal DSL effect.
deadly_tempest: // TODO: The "each player loses life equal to creatures they controlled" requires
enduring_vitality: // TODO: die-return-as-enchantment (Enduring cycle mechanic — zone-change
glimmer_lens: // TODO: For Mirrodin! + equipped attack trigger not expressible.
izoni_thousand_eyed: // TODO: Variable token count + sacrifice-other cost not expressible.
mana_leak: // TODO: "Counter unless controller pays {3}" — requires CounterUnlessPays effect.
nullmage_shepherd: // TODO: The activated ability cost requires tapping N other permanents you control
rings_of_brighthearth: // TODO: whenever you activate a non-mana ability, may pay {2} to copy it
smothering_tithe: // TODO: "that player may pay {2}, if they don't" — MayPayOrElse still a gap.
teferi_hero_of_dominaria: // TODO: TriggerEvent::WheneverYouDrawCard not in DSL — emblem trigger gap.
twilight_prophet: // TODO: Upkeep trigger conditioned on city's blessing, with drain-life based on
```

## Recent card-touching commits

```
1a04b73d scutemob-14: PB-CC-A — EffectAmount::PlayerCounterCount + Vishgraz deferred
7dcd6119 scutemob-13: PB-CC-C review fixes — defer Fuseling, ship engine variants for spell use cases (Option B per reviewer)
f965a783 scutemob-13: PB-CC-C — ModifyPowerDynamic + ModifyToughnessDynamic + Exuberant Fuseling CDA
f864bc25 scutemob-12: PB-CC-B review fixes — CR 121→122 citations and test library discriminators
b990b867 scutemob-12: PB-CC-B — TargetFilter.has_counter_type field + Armorcraft Judge ETB
0a46e1b4 merge: scutemob-11 — PB-CC-W: Mossborn Hydra Landfall counter-doubling wire-up — no engine changes
01c41cb9 scutemob-10: address /review findings — CR citation fixes + Blasphemous Edict + Vraska's Fall revert + Accursed Marauder Warrior
0f224094 scutemob-10: PB-SFT — add filter: Option<TargetFilter> to Effect::SacrificePermanents
3423b194 scutemob-11: fix CR citation in Mossborn Hydra (CR 121.2 → CR 122.1/122.6)
5237fcfb scutemob-11: PB-CC-W wire up Mossborn Hydra Landfall counter doubling
173fb5e5 W3: PB-N-L01 — reflow filter:None indentation in 5 card defs (CLOSED)
39b43f3a task scutemob-5: PB-T update card defs (14 cards)
872ea5d2 scutemob-4: PB-L — Landfall battlefield dispatch + stale-TODO sweep
1d7ecbf1 task scutemob-3: PB-P implement — EffectAmount::PowerOfSacrificedCreature (LKI capture-by-value)
b55ad321 scutemob-2: PB-D cards — 6 card defs use DamagedPlayer filter
0e5d7cf1 W6-prim: PB-N fix phase — 6 findings (F1+F2 HIGH, F3+F4+F5 MEDIUM, F6 LOW)
d343e1ba W6-prim: PB-N — SubtypeFilteredAttack + SubtypeFilteredDeath triggers
fc83d9d0 W6-prim: stale-TODO sweep — PB-N pre-launch (3 cards)
9c347754 W6-prim: PB-Q4 implement — EnchantTarget::Filtered + 4 land-aura cards
464d9e79 W6-prim: PB-Q close — 2 cards shipped, 4 parked, 4 micro-PBs reserved
880b7797 W6-prim: PB-Q implement — ChooseColor primitive (6 cards)
10411bd8 W6-prim: PB-X review fixes — Obelisk replacement form + CR citations + 3 integration tests
049b6802 W6-prim: PB-X implement — A-42 Tier 1 micro-PB
9dc9331a W6-prim: PB-S implement — GrantActivatedAbility (5 cards unblocked, 10 tests)
416c8a72 W6-prim: PB-M review fixes — Ancient Greenwarden legend rule, CardDef ETB test
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
