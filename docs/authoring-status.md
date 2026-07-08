<!-- DO NOT EDIT — regenerate via `python3 tools/authoring-report.py` -->

# Card Authoring Status — Canonical Report

**Generated:** 2026-07-08 02:22 UTC  
**Git:** `0f30d81e` on `feat/pb-ac3-dynamic-pt-count-amounts-cda-residual`  
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
| Clean (no TODO/ENGINE-BLOCKED, non-empty abilities)  — 54.4% | 951 | +5 |
| With TODO markers | 616 | -5 |
| Empty `abilities: vec![]` placeholders | 181 | · |
| Total TODO lines across all defs | 1,106 | -16 |

## Authoring activity (git, by window)

| Window | New files added | Existing files modified |
| --- | ---: | ---: |
| last 7 days | 0 | 481 |
| last 30 days | 0 | 481 |
| last 90 days | 14 | 589 |
| last 1 year | 1,773 | 1,236 |

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
| `combat-keyword` | 187 / 187 | 100% | 77 | 100 | 10 |
| `draw` | 163 / 169 | 96% | 66 | 72 | 25 |
| `token-create` | 145 / 155 | 94% | 20 | 60 | 65 |
| `land-etb-tapped` | 138 / 138 | 100% | 116 | 20 | 2 |
| `other` | 108 / 131 | 82% | 65 | 43 | 0 |
| `modal-choice` | 68 / 105 | 65% | 27 | 41 | 0 |
| `mana-land` | 92 / 92 | 100% | 74 | 15 | 3 |
| `body-only` | 55 / 70 | 79% | 23 | 10 | 22 |
| `removal-destroy` | 56 / 56 | 100% | 33 | 16 | 7 |
| `counters-plus` | 49 / 49 | 100% | 21 | 28 | 0 |
| `land-fetch` | 45 / 45 | 100% | 27 | 17 | 1 |
| `attack-trigger` | 6 / 34 | 18% | 2 | 4 | 0 |
| `death-trigger` | 34 / 34 | 100% | 18 | 15 | 1 |
| `mana-artifact` | 34 / 34 | 100% | 22 | 10 | 2 |
| `activated-tap` | 2 / 27 | 7% | 1 | 1 | 0 |
| `pump-buff` | 27 / 27 | 100% | 16 | 11 | 0 |
| `cant-restriction` | 25 / 25 | 100% | 16 | 9 | 0 |
| `removal-damage-target` | 23 / 23 | 100% | 6 | 12 | 5 |
| `activated-sacrifice` | 3 / 19 | 16% | 1 | 2 | 0 |
| `mana-creature` | 19 / 19 | 100% | 14 | 5 | 0 |
| `graveyard-recursion` | 18 / 18 | 100% | 7 | 11 | 0 |
| `removal-damage-each` | 17 / 17 | 100% | 10 | 7 | 0 |
| `counter` | 16 / 16 | 100% | 6 | 6 | 4 |
| `removal-exile` | 13 / 14 | 93% | 5 | 2 | 6 |
| `untap-phase` | 1 / 13 | 8% | 0 | 1 | 0 |
| `cost-reduction` | 12 / 12 | 100% | 4 | 0 | 8 |
| `opponent-punish` | 12 / 12 | 100% | 3 | 9 | 0 |
| `equipment` | 11 / 11 | 100% | 6 | 5 | 0 |
| `tutor` | 11 / 11 | 100% | 6 | 4 | 1 |
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
| OTHER (unclassified) | 654 | -3 |
| DSL gap (unspecified) | 140 | · |
| attack trigger (self / generic) | 28 | · |
| Cost::* missing variant | 22 | · |
| TriggerCondition::* missing variant | 19 | · |
| replacement effect missing | 18 | · |
| dynamic hexproof / protection | 17 | · |
| EffectAmount::* missing variant | 15 | -3 |
| sacrifice as cost | 15 | · |
| TargetFilter missing field | 12 | · |
| interactive / hidden-info choice | 11 | · |
| combat-damage-to-player trigger | 10 | · |
| opponent-action trigger | 9 | · |
| can't / must block-attack | 8 | · |
| can't be countered | 7 | · |
| no-maximum-hand-size | 7 | · |
| per-player effect dispatch | 6 | · |
| per-opponent upkeep | 6 | · |
| X-scaled tokens | 5 | -2 |
| devotion | 5 | · |
| count-threshold static | 5 | · |
| conditional static / grant | 5 | · |
| equipment grants ability | 5 | · |
| delayed triggers | 4 | · |
| untap-all / untap trigger | 4 | · |

_…and 33 more buckets totaling 69 lines._

### Raw OTHER samples (read these to design new classifier buckets)

Showing 12 of 654 
unclassified TODO lines. If two or three of these have a common theme, that's a 
new bucket to add to `TODO_BUCKETS` in `tools/authoring-report.py`. Sample is 
deterministic (sorted by slug).

```
abstergo_entertainment: // TODO: {3}, {T}, Exile Abstergo Entertainment: Return up to one target historic card
bonecrusher_giant: // TODO(2): Effect target is WRONG — should deal 2 damage to "that spell's controller"
dark_petition: // TODO: Condition::SpellMastery (2+ instant/sorcery in graveyard) not in DSL.
enduring_vitality: // TODO: die-return-as-enchantment (Enduring cycle mechanic — zone-change
glimmer_lens: // TODO: For Mirrodin! + equipped attack trigger not expressible.
ixhel_scion_of_atraxa: // TODO: Corrupted end-step trigger — per-opponent conditional exile + play-from-exile.
mana_vault: // ENGINE-BLOCKED: "{T}: Add {C}{C}{C}." — per W5 policy (KI-13 class), this mana ability's
open_the_vaults: // TODO(M10+): Add Aura placement choice so Auras can attach to valid targets.
roil_elemental: // TODO: Blocker — EffectDuration::WhileYouControlSource variant for "for as long as you
smothering_abomination: // TODO: "At the beginning of your upkeep, sacrifice a creature" — forced sacrifice not expressible.
teferi_master_of_time: // TODO: "−3: phases out" — no Effect::PhaseOut variant.
tyvar_jubilant_brawler: // TODO: static — creatures you control can activate abilities as though they had haste
```

## Recent card-touching commits

```
0f30d81e W6-prim: PB-AC3 card backfill — Keep Watch, Throne, Mirror Entity, Krenko, CDA residuals + tests
456a0bd7 W6-prim: PB-AC2 card review + real-card integration tests (closes MEDIUM #4)
507a476f W6-prim: PB-AC2 backfill — 12 clean + 8 partial card defs
34bee37c W6-prim: PB-AC1 backfill — re-author cards unblocked by untap/counter/once-per-turn
19b1f364 W6-prim: PB-AC1 implement — counter / untap / once-per-turn primitives
a1ed95a6 W5-cards: scutemob-42 — address 3 LOW review findings (batch 2)
2e9af171 W5-cards: scutemob-42 — re-author 12 stale-TODO cards (W-NOW-1 batch 2)
1f27f39c W6-prim: PB-AC0 — ETBTriggerFilter subtype/nontoken forwarding on creature-ETB path
c68695ea W5-cards: scutemob-40 — re-author 12 verified-stale-TODO cards
83695e65 scutemob-38: author Insatiable Avarice + Spree base-cost tests — LS-8
c3b0e399 scutemob-36: LS-6 review fixes — ETB test, CR 701.8 citations, test headers
5a8b404e W6-prim: PB-LS6 card defs — Sorin -6, Tamiyo -2, Hands of Binding freeze rider
27c1381b scutemob-28: PB-EWC-D — ObjectFilter::CreatureControlledByOfSubtype + bind_object_filter OwnedByOpponentsOf rebind
9eca57af scutemob-26: PB-XA2 — TargetFilter.is_blocking/is_tapped/is_untapped runtime predicates
8edf0e8a scutemob-25: PB-EAT — ReplacementModification::EntersAsAdditionalType (Master Biomancer Mutant half)
d7d8062e scutemob-24: PB-XA — card def cleanup + doc comment update
92f606d6 scutemob-23: review fix — correct ETB ordering caveat in comment
2ca4c307 scutemob-23: OOS-EWC-2 — Golgari Grave-Troll dynamic ETB counters
4d9862f6 scutemob-22: PB-XS-E — trigger-side exclude_self for "Whenever another permanent enters"
9b679fb8 scutemob-21: PB-XS — TargetFilter.exclude_self for "another target X"
37f77495 scutemob-20: PB-EWC — EntersWithCounters count u32→Box<EffectAmount> (CR 614.1c)
e7a9a16c scutemob-19: PB-LKI-Power — LKI source-power snapshot for WhenDies/WhenLeavesBattlefield (CR 603.10a)
f8d7cdf4 scutemob-18: PB-CD — counter-doubling replacement effects (CR 122.6/614.1)
34317614 feat(pb-lki-cc): add EffectAmount::CounterCountAtLastKnownInformation (disc 17) + LKI snapshot threading
4fde5d66 scutemob-16: PB-TS fix-phase — E1 Krenko sorcery-speed + C1 Chasm Skulker revert + OOS-TS-4 seed
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
