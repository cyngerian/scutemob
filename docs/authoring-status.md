<!-- DO NOT EDIT — regenerate via `python3 tools/authoring-report.py` -->

# Card Authoring Status — Canonical Report

**Generated:** 2026-08-02 09:25 UTC  
**Git:** `b76ef319` on `feat/cards-2-corpus-field-fidelity-audit-permanent-gate-mana-cost`  
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
| Clean (no TODO/ENGINE-BLOCKED, non-empty abilities)  — 63.0% | 1,135 | · |
| With TODO markers | 517 | · |
| Empty `abilities: vec![]` placeholders | 151 | · |
| Total TODO lines across all defs | 941 | · |

## Authoring activity (git, by window)

| Window | New files added | Existing files modified |
| --- | ---: | ---: |
| last 7 days | 0 | 76 |
| last 30 days | 57 | 2,943 |
| last 90 days | 57 | 2,955 |
| last 1 year | 1,830 | 3,366 |

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
| `W5-cards` | 35 |
| `W6-prim` | 18 |
| `chore` | 11 |
| `W1-Morph` | 3 |

**By month added:** 2026-02: 137, 2026-03: 172, 2026-04: 11, 2026-07: 1

## Coverage by authoring-plan group

"Clean" / "TODO" / "Empty" subdivide the *authored* count by file quality. 
Groups with high authored-but-not-clean ratios are TODO-debt — the cards exist but 
are blocked on engine primitives.

| Group | Auth / Total | % | Clean | TODO | Empty |
| --- | ---: | ---: | ---: | ---: | ---: |
| `combat-keyword` | 187 / 187 | 100% | 88 | 84 | 15 |
| `draw` | 164 / 169 | 97% | 78 | 69 | 17 |
| `token-create` | 148 / 155 | 95% | 86 | 46 | 16 |
| `land-etb-tapped` | 138 / 138 | 100% | 115 | 23 | 0 |
| `other` | 108 / 131 | 82% | 69 | 31 | 8 |
| `modal-choice` | 73 / 105 | 70% | 37 | 24 | 12 |
| `mana-land` | 92 / 92 | 100% | 65 | 26 | 1 |
| `body-only` | 64 / 70 | 91% | 38 | 10 | 16 |
| `removal-destroy` | 56 / 56 | 100% | 35 | 17 | 4 |
| `counters-plus` | 49 / 49 | 100% | 25 | 19 | 5 |
| `land-fetch` | 45 / 45 | 100% | 27 | 14 | 4 |
| `attack-trigger` | 19 / 34 | 56% | 16 | 2 | 1 |
| `death-trigger` | 34 / 34 | 100% | 20 | 9 | 5 |
| `mana-artifact` | 34 / 34 | 100% | 22 | 10 | 2 |
| `activated-tap` | 9 / 27 | 33% | 8 | 0 | 1 |
| `pump-buff` | 27 / 27 | 100% | 17 | 7 | 3 |
| `cant-restriction` | 25 / 25 | 100% | 16 | 5 | 4 |
| `removal-damage-target` | 23 / 23 | 100% | 11 | 10 | 2 |
| `activated-sacrifice` | 8 / 19 | 42% | 7 | 1 | 0 |
| `mana-creature` | 19 / 19 | 100% | 14 | 5 | 0 |
| `graveyard-recursion` | 18 / 18 | 100% | 8 | 6 | 4 |
| `removal-damage-each` | 17 / 17 | 100% | 12 | 4 | 1 |
| `counter` | 16 / 16 | 100% | 8 | 5 | 3 |
| `removal-exile` | 14 / 14 | 100% | 5 | 5 | 4 |
| `untap-phase` | 6 / 13 | 46% | 5 | 0 | 1 |
| `cost-reduction` | 12 / 12 | 100% | 12 | 0 | 0 |
| `opponent-punish` | 12 / 12 | 100% | 5 | 2 | 5 |
| `equipment` | 11 / 11 | 100% | 6 | 5 | 0 |
| `tutor` | 11 / 11 | 100% | 9 | 1 | 1 |
| `removal-bounce` | 10 / 10 | 100% | 5 | 4 | 1 |
| `static-enchantment` | 1 / 8 | 12% | 1 | 0 | 0 |
| `discard-effect` | 4 / 7 | 57% | 4 | 0 | 0 |
| `scry-surveil` | 7 / 7 | 100% | 4 | 3 | 0 |
| `aura` | 6 / 6 | 100% | 3 | 2 | 1 |
| `etb-trigger` | 6 / 6 | 100% | 1 | 4 | 1 |
| `lifedrain` | 6 / 6 | 100% | 3 | 1 | 2 |
| `sacrifice-outlet` | 6 / 6 | 100% | 6 | 0 | 0 |
| `lifegain` | 5 / 5 | 100% | 3 | 0 | 2 |
| `mana-other` | 5 / 5 | 100% | 3 | 2 | 0 |
| `removal-minus` | 4 / 4 | 100% | 3 | 0 | 1 |
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

#### `activated-sacrifice` — 8 / 19 (42%), authored split: 7 clean / 1 todo / 0 empty — **unwritten**

| Card | Slug | Bucket |
| --- | --- | --- |
| Altar of Dementia | `altar_of_dementia` | clean |
| An Offer You Can't Refuse | `an_offer_you_cant_refuse` | clean |
| Birthing Pod | `birthing_pod` | clean |
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
| OTHER (unclassified) | 577 | · |
| DSL gap (unspecified) | 122 | · |
| attack trigger (self / generic) | 23 | · |
| TriggerCondition::* missing variant | 17 | · |
| dynamic hexproof / protection | 15 | · |
| replacement effect missing | 14 | · |
| Cost::* missing variant | 13 | · |
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

_…and 26 more buckets totaling 44 lines._

### Raw OTHER samples (read these to design new classifier buckets)

Showing 12 of 577 
unclassified TODO lines. If two or three of these have a common theme, that's a 
new bucket to add to `TODO_BUCKETS` in `tools/authoring-report.py`. Sample is 
deterministic (sorted by slug).

```
abstergo_entertainment: // TODO: {3}, {T}, Exile Abstergo Entertainment: Return up to one target historic card
bloodghast: // TODO: Oracle says "you may return" — currently non-optional (bot always returns).
deep_gnome_terramancer: // TODO: "lands enter under opponent's control without being played" trigger condition
exuberant_fuseling: // TODO: "whenever another creature or artifact you control is put into a graveyard
goblin_king: // TODO: AllCreaturesWithSubtype includes Goblin King itself — "other" semantics
joraga_treespeaker: // TODO: Level up mechanic not in DSL — no LevelUp keyword or level-based ability gating.
marisi_breaker_of_the_coil: // TODO: "Your opponents can't cast spells during combat" — phase-scoped CantCast not in DSL.
overwhelming_stampede: // TODO: Spell effect — grant trample and +X/+X to all creatures you control until end
sakashimas_student: // TODO: "enter as a copy of any creature, except it's also a Ninja" — needs
sorin_imperious_bloodlord: // TODO: "You may sacrifice a Vampire. When you do, [effects]" — optional sacrifice
teferis_protection: // TODO: "All permanents you control phase out" — Effect::PhaseOut for all controller permanents.
tyvar_jubilant_brawler: // TODO: static — creatures you control can activate abilities as though they had haste
```

## ⚠ Completeness-marker drift

10 defs whose `completeness:` marker contradicts their comments. The marker is authoritative (it is what `validate_deck` reads), so fix whichever is stale.

- `ashnods_altar` — marked partial but has no TODO / ENGINE-BLOCKED comment
- `boggart_shenanigans` — marked partial but has no TODO / ENGINE-BLOCKED comment
- `contaminant_grafter` — marked partial but has no TODO / ENGINE-BLOCKED comment
- `emeria_the_sky_ruin` — marked partial but has no TODO / ENGINE-BLOCKED comment
- `grateful_apparition` — marked partial but has no TODO / ENGINE-BLOCKED comment
- `hullbreaker_horror` — marked partial but has no TODO / ENGINE-BLOCKED comment
- `phyrexian_tower` — marked partial but has no TODO / ENGINE-BLOCKED comment
- `shambling_ghast` — marked partial but has no TODO / ENGINE-BLOCKED comment
- `temple_of_the_dragon_queen` — marked partial but has no TODO / ENGINE-BLOCKED comment
- `thrasios_triton_hero` — marked partial but has no TODO / ENGINE-BLOCKED comment

## Recent card-touching commits

```
b76ef319 scutemob-181: CARDS-2 — SR-37 docs, two honest demotions, seeds, close-out
b7a46cb3 scutemob-181: CARDS-2 — repair all 45 printed-field mismatches; gate green
99d54ed5 scutemob-179: CARDS-1 — author the CR 702.6a equip target into all 17 equip defs (OOS-M11-10)
869f09d6 scutemob-168: PB-DX4 fix cycle — 2 HIGH / 5 MEDIUM / 6 LOW from pb-review-DX4.md, all 13 applied
e658c9d8 scutemob-168: PB-DX4 — triage the 97-entry decision BASELINE against oracle text (OOS-DP10-8)
39d4902c scutemob-166: PB-DX3b fix cycle — apply review findings (5 MEDIUM / 7 LOW)
bc035684 scutemob-166: PB-DX3b implement — OOS-DX3-1 bucket, 4 fixes / 3 dated defers
8e336544 scutemob-164: PB-DX3 fix cycle -- apply review findings (1 MEDIUM / 5 LOW)
f3e92ecc scutemob-164: PB-DX3 implement — two stale blocker notes closed (OOS-DP6-3)
95c35f7a W6-prim: scutemob-160 — PB-DX1 review fixes
03053182 W6-prim: scutemob-160 — PB-DX1 phase 8: card def dispositions (§6.3, §6.4)
460e7f4e W6-prim: scutemob-154 -- PB-DP6 phase 1: queue-time intervening-if gate (CR 603.4)
5c463339 W6-prim: scutemob-152 -- PB-DP4 engine changes: attack tax debit + payment deadline (DP-10/DP-11)
36e64283 W6-prim: scutemob-145 — PB-RS3 /review nits: correct two overstated claims
83e0f4e1 W6-prim: scutemob-145 — PB-RS3 fix cycle: all 7 review findings (0 HIGH, 3 MEDIUM, 4 LOW)
95626742 W6-prim: scutemob-145 — PB-RS3 card-def roster: 3 flips + 1 integrity repair
7d932614 W6-prim: scutemob-144 — PB-RS2 fix cycle: apply all 12 review findings (0 HIGH, 5 MEDIUM, 7 LOW)
48cec52a W6-prim: scutemob-144 — PB-RS2: CR 119.4 combined-life fix, birthing_pod flip, roster sweep, simulator
e29dd99f W6-prim: PB-OS11 review-fix — flip gemstone_array + druids_repository known_wrong→Complete
69768559 W6-prim: PB-OS11 — final PB-OS batch: RemoveCounter mana-ability lowering + batch filtered-attack trigger
d876d19b W6-prim: PB-OS10 — inter-target distinctness + Jitte any-recipient combat trigger
63335b8d W6-prim: PB-OS9 — Condition::YouControlYourCommander (OOS-EF3b-1); PROTOCOL 24 / HASH 61
63148132 W6-prim: PB-OS8 — Effect::LookAtTopThenPlace + TargetFilter.min_cmc_amount (OOS-EF10-1 + OS6-deferred-(d))
2beaba4f W6-prim: PB-OS7 — author silumgar_the_drifting_death.rs (Complete)
bd15b45b W6-prim: PB-OS6 review LOW — soften delver 'strictly beneficial' comment wording
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
