<!-- DO NOT EDIT — regenerate via `python3 tools/authoring-report.py` -->

# Card Authoring Status — Canonical Report

**Generated:** 2026-09-04 12:53 UTC  
**Git:** `650a6cfb` on `feat/pb-dx36-a-general-whendealsdamage-trigger-damage-dealt-effec`  
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
| Clean (no TODO/ENGINE-BLOCKED, non-empty abilities)  — 63.2% | 1,139 | · |
| With TODO markers | 517 | · |
| Empty `abilities: vec![]` placeholders | 147 | · |
| Total TODO lines across all defs | 916 | · |

## Authoring activity (git, by window)

| Window | New files added | Existing files modified |
| --- | ---: | ---: |
| last 7 days | 0 | 26 |
| last 30 days | 0 | 156 |
| last 90 days | 57 | 2,947 |
| last 1 year | 1,830 | 3,370 |

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
| `draw` | 164 / 169 | 97% | 79 | 69 | 16 |
| `token-create` | 148 / 155 | 95% | 87 | 46 | 15 |
| `land-etb-tapped` | 138 / 138 | 100% | 115 | 23 | 0 |
| `other` | 108 / 131 | 82% | 71 | 30 | 7 |
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
| OTHER (unclassified) | 565 | · |
| DSL gap (unspecified) | 116 | · |
| attack trigger (self / generic) | 23 | · |
| TriggerCondition::* missing variant | 17 | · |
| dynamic hexproof / protection | 15 | · |
| replacement effect missing | 14 | · |
| Cost::* missing variant | 13 | · |
| EffectAmount::* missing variant | 11 | · |
| combat-damage-to-player trigger | 10 | · |
| interactive / hidden-info choice | 10 | · |
| sacrifice as cost | 8 | · |
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
curiosity: // TODO: costless "you may draw a card" — CR 603.3c optionality with NO cost
everflowing_chalice: // TODO: "This artifact enters with a charge counter on it for each time it was kicked." —
glimmer_lens: // TODO: the attack trigger only — "For Mirrodin!" is expressible and unauthored
jeskas_will: // TODO: Mode 2 needs impulse-draw (exile top 3, play this turn).
marionette_apprentice: // ENGINE-BLOCKED: "Whenever another creature or artifact you control dies" — there is no
pact_of_negation: // TODO: Counter target spell + delayed upkeep trigger "pay {3}{U}{U} or lose the game."
sarkhan_fireblood: // TODO: "Any combination of colors" + Dragon-only restriction not in DSL.
sorin_imperious_bloodlord: // TODO: Interactive hand selection by creature subtype ("Vampire creature card from
teferis_protection: // TODO: "All permanents you control phase out" — Effect::PhaseOut for all controller permanents.
tyvar_jubilant_brawler: // TODO: static — creatures you control can activate abilities as though they had haste
```

## ⚠ Completeness-marker drift

16 defs whose `completeness:` marker contradicts their comments. The marker is authoritative (it is what `validate_deck` reads), so fix whichever is stale.

- `ashnods_altar` — marked partial but has no TODO / ENGINE-BLOCKED comment
- `birchlore_rangers` — marked partial but has no TODO / ENGINE-BLOCKED comment
- `boggart_shenanigans` — marked partial but has no TODO / ENGINE-BLOCKED comment
- `braided_net` — marked partial but has no TODO / ENGINE-BLOCKED comment
- `chord_of_calling` — marked Complete but has a TODO / ENGINE-BLOCKED comment
- `contaminant_grafter` — marked partial but has no TODO / ENGINE-BLOCKED comment
- `emeria_the_sky_ruin` — marked partial but has no TODO / ENGINE-BLOCKED comment
- `encroaching_dragonstorm` — marked partial but has no TODO / ENGINE-BLOCKED comment
- `grateful_apparition` — marked partial but has no TODO / ENGINE-BLOCKED comment
- `hullbreaker_horror` — marked partial but has no TODO / ENGINE-BLOCKED comment
- `marisi_breaker_of_the_coil` — marked partial but has no TODO / ENGINE-BLOCKED comment
- `phyrexian_tower` — marked partial but has no TODO / ENGINE-BLOCKED comment
- `qarsi_sadist` — marked partial but has no TODO / ENGINE-BLOCKED comment
- `temple_of_the_dragon_queen` — marked partial but has no TODO / ENGINE-BLOCKED comment
- `the_reaver_cleaver` — marked partial but has no TODO / ENGINE-BLOCKED comment
- `thrasios_triton_hero` — marked partial but has no TODO / ENGINE-BLOCKED comment

## Recent card-touching commits

```
1ab2bef8 scutemob-228: PB-DX36 — a sentinel my own sweep missed, and four ratchets the step-8 tests surfaced
d5fa56ba scutemob-228: PB-DX36 census follow-through — two blocker notes this batch FALSIFIES, narrowed in place
2aa5e08f scutemob-228: PB-DX36 coverage — 1,138 -> 1,139 / 1,803 = 63.2%, the ONE flip named before regeneration (exalted_angel)
70cba583 scutemob-228: PB-DX36 — correct the CR cite the task brief supplied, and two card-def judgment calls
b1757a5b scutemob-228: PB-DX36 steps 1-7 — DamageRecipient, WhenDealsDamage, and the combat/noncombat "deals damage" trigger dispatch unification
143bfcde scutemob-227: PB-DX35 /review fix cycle — all 9 findings taken, four of them gate defeats the reviewer PROVED by execution
7e4eb0d9 scutemob-227: PB-DX35 Half B — Effect::LookAtTopThenPlace.optional becomes a real CR 118.12 player decision (OOS-DX4-5), asked through the same EffectChoiceQuestion::ChooseObject { count: 1, up_to: true } PB-DX28 built. Sorts top_ids ascending before deciding the winner (Zone::top_n is top-first, the reverse of ascending-id order); no determined-answer short-circuit for a lone candidate, since up_to: true makes declining a real second answer even then. Six comments corrected (two beyond the plan's five, found by the B6 consumer enumeration: replay_harness.rs's golden-script driver and the TUI's log formatter both claimed ChooseObject named only public objects). decision_site_walk's compound look_at_top_or_route row split rather than residual-noted, since leaving it compound would have silently un-flagged RevealAndRoute's still-engine-made CR 401.4 order choice; MAX_AUTO_CHOSEN_COMPLETE_UNION 72->67, five BASELINE entries removed, OOS-DX35-1 filed for the RevealAndRoute residual and OOS-DX4-5 closed in the registry. decision_coverage.rs's row_id_for needed real disambiguation logic, not just a doc fix: ChooseObject is now asked by two unrelated primitives sharing one wire shape, told apart by candidate ZONE (library = PB-DX35, battlefield/graveyard = PB-DX28's pre-existing untargeted choice). Three-channel reachability (LocalGame/ HumanChoice, POST /api/game/action, the bot path) each proven with a genuine decline, asserted by resolution effect — HTTP channel via Satyr Wayfinder (a real "which-of-four" choice, seed found by an executed scan) since its four-card dig exercises the choice more fully than Risen Reef's single-card one. Seven pre-existing pb_os8 tests needed execute_effect -> execute_effect_with_default_choices repair, a ripple the plan did not name, reproduced red before the fix. HASH 82 / PROTOCOL 41 both unmoved as predicted, gate-executed. 15 new tests (9 engine + 3 simulator + 3 HTTP), 0 renames, 0 removals; full workspace 5,076 -> 5,091 / 0 / 5, 63 targets. 0 Completeness marker moves (5 card-def edits, comment-only, verified line-by-line). Revert matrix: t3/t6/t7/t8 + all 6 channel probes redden under a full revert; t5 additionally discriminates a narrower sort-only revert; t1/t2/t4 are stated CONTROLS. Full record appended to memory/primitives/pb-DX35-execution-notes.md §B (§0 untouched).
dfd6e1ce scutemob-227: PB-DX35 Half A — re-observe every standing gate the batch's own card-def flips and refactor moved: SR-25 bare-lookup ceiling 75->72 (trigger_modal_plan consolidation), unordered-container ceiling 6->8 (t9's lookup-only HashMap, category (a)), card-defs fmt fix, decision_gate's MAX_AUTO_CHOSEN_COMPLETE_UNION 71->72 and BASELINE (Shambling Ghast added, modal_trigger row), canonical_walk_reproduces_pb_dp8_roster and pb_dp8_trigger_target_choice's roster floor 60->59 (retreat_to_kazandu's target left the flat targets list), completeness_deviation_scan's marker floor 666->665 and RECORDED_BASELINE_POPULATION 45->47 (two new entries), pb_dx4_baseline_triage's stale "Shambling Ghast must not be Complete" pin removed and disclosed, pb_dx32_fuzz_output's CORPUS_COMPLETE 1137->1138; all re-derived by executing the failing gate's own output, never computed
ab6d8859 scutemob-227: PB-DX35 Half A card defs — shambling_ghast/retreat_to_kazandu/ retreat_to_coralhelm re-shaped into ModeSelection.mode_targets (shambling_ghast partial -> Complete); hullbreaker_horror/glissa_sunslayer/junji_the_midnight_sky markers re-adjudicated to name the registry-vs-runtime index-space blocker (OOS-DX35-1, not fixed); felidar_retreat noted as out of population
b72b8c80 scutemob-225: PB-DX18 /review fix cycle — all 15 findings taken, none declined
877510c5 scutemob-225: PB-DX18 — the phantom shuffles really shuffle (OOS-DP2-7), plus the fixture repairs
e7dee121 scutemob-225: PB-DX18 — CR 702.47a splice targets, and the golden script that said bestow and did not
0be8d904 scutemob-222: PB-DX20b -- EnchantFilter gains the OR over card TYPES, and the two arithmetics become one
e524f676 scutemob-217: PB-DX45 /review fix cycle -- all 7 findings taken
6af13425 scutemob-217: PB-DX45 -- the CR 118.12 suspension, the wire bump, and the policy ruling
40b1e610 scutemob-216: PB-DX15a /review fix cycle 2 -- probes for the three uncovered APNAP sites, and five failures my own fix cycle introduced
4c2a0afd scutemob-216: PB-DX15a /review fix cycle -- the HIGH is a regression I introduced, and two of my own claims did not survive
7c435919 scutemob-213: PB-DX43 S1-S4 -- CR 305.6/305.7 intrinsic land mana abilities
2ca6a741 scutemob-211: PB-DX29 /review H2 — the renumbering orphaned 30 in-source cites and the note asserted the opposite
753afb9c scutemob-211: PB-DX29 part B1 — the provider learns the seven remaining cast-side cost kinds
de75b78d scutemob-211: PB-DX29 — marker/cost roster gate for all eight keyword-carried costs, three card-def repairs, OOS-M11-10 collision resolved
9f3e41c0 scutemob-210: PB-DX28 AC4 — retire the allowlist entries, prove the scan still reddens
1de151c7 scutemob-210: PB-DX28 — migrate the 18th member (Connive // Concoct), found by the batch's own gate
1babe026 scutemob-210: PB-DX28 part 2 — the untargeted-choice channel (OOS-DX4-6)
6aeb2008 scutemob-210: PB-DX28 part 1 — the owner axis (OOS-DX4-1) + EffectTarget::DamagedPlayer
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
