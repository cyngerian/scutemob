<!-- DO NOT EDIT — regenerate via `python3 tools/authoring-report.py` -->

# Card Authoring Status — Canonical Report

**Generated:** 2026-08-14 01:05 UTC  
**Git:** `2cbd5996` on `feat/pb-dx27-stale-blocker-note-sweep-wrong-oracle-register-oos-c`  
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
| Clean (no TODO/ENGINE-BLOCKED, non-empty abilities)  — 63.1% | 1,137 | +4 |
| With TODO markers | 519 | · |
| Empty `abilities: vec![]` placeholders | 147 | -4 |
| Total TODO lines across all defs | 918 | -15 |

## Authoring activity (git, by window)

| Window | New files added | Existing files modified |
| --- | ---: | ---: |
| last 7 days | 0 | 44 |
| last 30 days | 57 | 1,772 |
| last 90 days | 57 | 2,946 |
| last 1 year | 1,830 | 3,369 |

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
| `draw` | 164 / 169 | 97% | 78 | 70 | 16 |
| `token-create` | 148 / 155 | 95% | 87 | 46 | 15 |
| `land-etb-tapped` | 138 / 138 | 100% | 115 | 23 | 0 |
| `other` | 108 / 131 | 82% | 72 | 29 | 7 |
| `modal-choice` | 73 / 105 | 70% | 37 | 24 | 12 |
| `mana-land` | 92 / 92 | 100% | 65 | 26 | 1 |
| `body-only` | 64 / 70 | 91% | 38 | 10 | 16 |
| `removal-destroy` | 56 / 56 | 100% | 35 | 17 | 4 |
| `counters-plus` | 49 / 49 | 100% | 25 | 19 | 5 |
| `land-fetch` | 45 / 45 | 100% | 28 | 13 | 4 |
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
| OTHER (unclassified) | 565 | -12 |
| DSL gap (unspecified) | 118 | +1 |
| attack trigger (self / generic) | 23 | · |
| TriggerCondition::* missing variant | 17 | · |
| dynamic hexproof / protection | 15 | · |
| replacement effect missing | 14 | · |
| Cost::* missing variant | 13 | · |
| EffectAmount::* missing variant | 11 | -1 |
| combat-damage-to-player trigger | 10 | · |
| interactive / hidden-info choice | 10 | · |
| sacrifice as cost | 8 | -3 |
| can't / must block-attack | 7 | · |
| can't be countered | 7 | · |
| opponent-action trigger | 7 | · |
| TargetFilter missing field | 6 | · |
| per-opponent upkeep | 6 | · |
| conditional static / grant | 5 | · |
| delayed triggers | 5 | · |
| untap-all / untap trigger | 4 | · |
| noncombat-damage prevent | 4 | · |
| ETB choice | 4 | · |
| impulse draw | 4 | · |
| CDA / dynamic P/T | 4 | · |
| devotion | 4 | · |
| per-player effect dispatch | 3 | · |

_…and 26 more buckets totaling 44 lines._

### Raw OTHER samples (read these to design new classifier buckets)

Showing 12 of 565 
unclassified TODO lines. If two or three of these have a common theme, that's a 
new bucket to add to `TODO_BUCKETS` in `tools/authoring-report.py`. Sample is 
deterministic (sorted by slug).

```
abstergo_entertainment: // TODO: {3}, {T}, Exile Abstergo Entertainment: Return up to one target historic card
blood_seeker: // TODO: "that player" — effect should target the entering creature's controller specifically,
curiosity: // TODO(PB-37): approximation — oracle says "an opponent" but
everflowing_chalice: // TODO: "This artifact enters with a charge counter on it for each time it was kicked." —
glimmer_lens: // TODO: still genuinely blocked — "Whenever equipped creature and at least one other
jeskas_will: // TODO: Mode 1 needs mana-scaled-by-opponent-hand-count.
mardu_ascendancy: // TODO: Nontoken filter not yet in DSL for attack triggers — over-triggers on token
overwhelming_stampede: // TODO: Spell effect — grant trample and +X/+X to all creatures you control until end
sarkhan_fireblood: // TODO: Optional discard-then-draw not in DSL. Using Nothing to avoid free draw.
sorin_imperious_bloodlord: // TODO: "You may sacrifice a Vampire. When you do, [effects]" — optional sacrifice
teferi_temporal_archmage: // TODO: Emblem creation for "activate loyalty at instant speed" not in DSL.
tymna_the_weaver: // ENGINE-BLOCKED: the life payment and draw count both scale with the number of
```

## ⚠ Completeness-marker drift

19 defs whose `completeness:` marker contradicts their comments. The marker is authoritative (it is what `validate_deck` reads), so fix whichever is stale.

- `ashnods_altar` — marked partial but has no TODO / ENGINE-BLOCKED comment
- `birchlore_rangers` — marked partial but has no TODO / ENGINE-BLOCKED comment
- `boggart_shenanigans` — marked partial but has no TODO / ENGINE-BLOCKED comment
- `braided_net` — marked partial but has no TODO / ENGINE-BLOCKED comment
- `chord_of_calling` — marked Complete but has a TODO / ENGINE-BLOCKED comment
- `contaminant_grafter` — marked partial but has no TODO / ENGINE-BLOCKED comment
- `emeria_the_sky_ruin` — marked partial but has no TODO / ENGINE-BLOCKED comment
- `encroaching_dragonstorm` — marked partial but has no TODO / ENGINE-BLOCKED comment
- `grateful_apparition` — marked partial but has no TODO / ENGINE-BLOCKED comment
- `green_suns_zenith` — marked Complete but has a TODO / ENGINE-BLOCKED comment
- `hullbreaker_horror` — marked partial but has no TODO / ENGINE-BLOCKED comment
- `marisi_breaker_of_the_coil` — marked partial but has no TODO / ENGINE-BLOCKED comment
- `phyrexian_tower` — marked partial but has no TODO / ENGINE-BLOCKED comment
- `qarsi_sadist` — marked partial but has no TODO / ENGINE-BLOCKED comment
- `shambling_ghast` — marked partial but has no TODO / ENGINE-BLOCKED comment
- `temple_of_the_dragon_queen` — marked partial but has no TODO / ENGINE-BLOCKED comment
- `the_reaver_cleaver` — marked partial but has no TODO / ENGINE-BLOCKED comment
- `thrasios_triton_hero` — marked partial but has no TODO / ENGINE-BLOCKED comment
- `vampire_gourmand` — marked partial but has no TODO / ENGINE-BLOCKED comment

## Recent card-touching commits

```
2b485ccc scutemob-209: PB-DX27 rider OOS-ADJ-7 — Blood Moon strips land types, never card types
f1b81bfe scutemob-209: PB-DX27 sweep repairs — 4 stale blocker notes verified and closed
3390b6a9 scutemob-209: PB-DX27 sweep-repairs batch B — 5 stale blocker notes refuted and closed
429928d5 scutemob-209: PB-DX27 — wrong-oracle register (OOS-CARDS2-10) + the three OOS-CARDS2-11 headline items
3d5db7b2 scutemob-206: PB-DX26 fix cycle — all 18 review findings taken (1 HIGH / 6 MEDIUM / 11 LOW)
72ad0f93 scutemob-206: PB-DX26 — the equip surface, one link earlier
32373601 scutemob-205: PB-DX25c fix cycle — take all 22 review findings (0 HIGH / 5 MEDIUM / 17 LOW)
557ef5ce scutemob-205: PB-DX25c stage 2 — fixture repairs, new probes, roster/gate, HASH bump, revert matrix, seed close-out (closes OOS-DX25b-3)
a275a949 scutemob-204: PB-DX25b fix cycle — take all 12 findings (1 HIGH / 5 MEDIUM / 6 LOW)
cadb346b scutemob-202: PB-DX24 fix cycle — F12, drop stray blank line in nether_traitor.rs
0ca69b6b scutemob-202: PB-DX24 stage 7 — nether_traitor comment records what the engine now reads
789867db scutemob-201: PB-DX23 Stage 6 — the auto-discharge gets a row, and seeds
f573ef21 scutemob-200: PB-DX21 fix cycle — take M1-M5, M7, L1-L5, L7-L8 (M6/L6 coordinator's)
91e1f6b4 scutemob-200: PB-DX21 Stage 5 — comments and docs that PB-DX21 made wrong
a878ca26 scutemob-189: SIM-6 — activation-cost payment channel (sacrifice + discard) end to end
569087e6 scutemob-184: close OOS-DX19-1 for real, and gate it so the claim stays true
697606a6 scutemob-184: apply the rest of the review — devilish_valet probe, docs, seeds
79b94a58 scutemob-184: PB-DX19 — discriminating tests, each watched failing by revert
a0d977e5 scutemob-184: PB-DX19 — close OOS-SIM2-6 and fold in OOS-SIM2-5
a2fef7cd scutemob-178: UI-2 stage 1/2 — provider builds an AdditionalCostPlan; params defaults it; auto-tap pays Squad
2c7272fd scutemob-181: CARDS-2 fourth fix cycle — the closed-vs-not-firing overclaim, and two uncovered repairs
23b75885 scutemob-181: CARDS-2 third fix cycle — merge SIM-1, add R8, close the reviewer's list
0abb41db scutemob-181: CARDS-2 second fix cycle — the batch's own worst moment, documented
50f26048 scutemob-181: CARDS-2 fix cycle — the reviewer found the sharpest thing in the batch
b76ef319 scutemob-181: CARDS-2 — SR-37 docs, two honest demotions, seeds, close-out
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
