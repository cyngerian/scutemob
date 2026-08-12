# PB-DX26 — fail-before evidence (executed 2026-08-11)

> Every claim below is a captured run, not an argument. Two kinds of evidence:
> the **pre-fix measurement** (the corpus as PB-DX26 found it) and the **revert
> matrix** (each new gate/probe watched failing when the thing it guards is undone).

## 1. Pre-fix measurement

`core::pb_dx26_attach_keyword_roster`, run against the corpus BEFORE any card-def
edit. The headline numbers this batch rests on, measured by enumerating
`all_cards()` (SR-36), never by grep:

* **R1 = 21** defs carry `AbilityDefinition::Keyword(KeywordAbility::Equip)` — the
  seed's figure, confirmed by the `all_cards()` method. R1 PASSED pre-fix: the
  population was already right; it was the *ability* that was missing.
* **R2 = 21 of 21** had **no** activated ability reaching `Effect::AttachEquipment`.
  That is the whole defect: not one of the 21 printed equip lines existed at runtime.
* **R3 = 10** of them were deck-legal `Completeness::Complete`.
* **R4 (the type-line inverse census) found 24** with no attach path at all, i.e.
  **three defs beyond the 21** — `Lizard Blades` (a false positive: its attach is
  synthesised from `AbilityDefinition::Reconfigure` by `replay_harness.rs:4049`, and
  the census now counts that as reachable) and **`Quietus Spike` + `Sting, the
  Glinting Dagger`**, which print "Equip {N}" and carry **neither the marker nor the
  ability**, so no keyword-derived roster — the seed's grep or R1's `all_cards()`
  walk alike — could ever see them. This is the dispatch-hygiene-6 payoff: the
  brief's site list was a floor. Both are `Completeness::Inert` with
  `abilities: vec![]`, so the deck-legal blast radius is **0**; filed as `OOS-DX26-1`
  and pinned as R4's residual with the excusal itself asserted.
* **R5 (Fortification) PASSED**: `Darksteel Garrison` did have an activated
  `AttachFortification` ability. Its defect (`OOS-CARDS1-1`) was one link later — the
  ability existed and declared no target, which is why R5 is green here and the
  strengthened `t7b` requirement pin is what catches it.
* **R6 FAILED on its first run, and the failure was the gate working**: the pin was
  hand-written as `(7, 3)` and the enum's real nesting-site count is `(8, 2)`. The
  pinned count is now the executed measurement, not the author's transcription.

Verbatim:

```
running 6 tests
R6 measured Effect nesting sites: Box<Effect>=8, Vec<Effect>=2

thread 'pb_dx26_attach_keyword_roster::r6_effect_nesting_sites_are_pinned' (3085711) panicked at crates/engine/tests/core/pb_dx26_attach_keyword_roster.rs:412:5:
assertion `left == right` failed: The `Effect` enum's nesting sites changed (found Box<Effect>=8, Vec<Effect>=2; expected 7 and 3). `contains_attach` in this file walks them by hand: Conditional{if_true,if_false}, Repeat{effect}, ForEach{effect}, MayPayOrElse{or_else}, MayPayThenEffect{then}, CoinFlip{on_win,on_lose} (Box) and Sequence(..), Choose{choices} (Vec). Add the new site to `contains_attach` and re-pin this count, or every attach census in this file silently stops seeing effects nested inside it.
  left: (8, 2)
 right: (7, 3)
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
test pb_dx26_attach_keyword_roster::r6_effect_nesting_sites_are_pinned ... FAILED
R5 measured Fortification-subtyped population (1) = {"Darksteel Garrison"}
R2 measured marker-without-ability set = {"Blackblade Reforged", "Blade of the Bloodchief", "Bone Saw", "Commander's Plate", "Empyrial Plate", "Glimmer Lens", "Illusionist's Bracers", "Kite Shield", "Mask of Memory", "Paradise Mantle", "Sword of Body and Mind", "Sword of Feast and Famine", "Sword of Light and Shadow", "Sword of Sinew and Steel", "Sword of Truth and Justice", "Sword of War and Peace", "Sword of the Animist", "Sword of the Paruns", "The Reaver Cleaver", "Umbral Mantle", "Umezawa's Jitte"}

thread 'pb_dx26_attach_keyword_roster::r2_every_equip_marker_def_has_a_reachable_equip_ability' (3085707) panicked at crates/engine/tests/core/pb_dx26_attach_keyword_roster.rs:206:5:
PB-DX26 / OOS-CARDS1-3: 21 def(s) print 'Equip {N}' (they carry `AbilityDefinition::Keyword(KeywordAbility::Equip)`) but have NO activated ability whose effect reaches `Effect::AttachEquipment`. `keyword_registry.rs`'s `K::Equip` arm is a `KeywordHandling::Marker` — it synthesises nothing — so for these defs there is no ability for the provider to offer, no index for a client to name, and no `Command::ActivateAbility` that could reach one. The printed ability does not exist. Author it (see `skullclamp.rs` / `bone_saw.rs`).
Violations: {"Blackblade Reforged", "Blade of the Bloodchief", "Bone Saw", "Commander's Plate", "Empyrial Plate", "Glimmer Lens", "Illusionist's Bracers", "Kite Shield", "Mask of Memory", "Paradise Mantle", "Sword of Body and Mind", "Sword of Feast and Famine", "Sword of Light and Shadow", "Sword of Sinew and Steel", "Sword of Truth and Justice", "Sword of War and Peace", "Sword of the Animist", "Sword of the Paruns", "The Reaver Cleaver", "Umbral Mantle", "Umezawa's Jitte"}
R4 measured Equipment-subtyped population (42) = {"Accorder's Shield", "Argentum Armor", "Basilisk Collar", "Batterskull", "Blackblade Reforged", "Blade of the Bloodchief", "Bone Saw", "Cathar's Shield", "Commander's Plate", "Cryptic Coat", "Diamond Pick-Axe", "Empyrial Plate", "Glimmer Lens", "Hammer of Nazahn", "Helm of the Host", "Illusionist's Bracers", "Kite Shield", "Lightning Greaves", "Lizard Blades", "Mask of Memory", "Paradise Mantle", "Quietus Spike", "Shadowspear", "Skullclamp", "Spidersilk Net", "Sting, the Glinting Dagger", "Swiftfoot Boots", "Sword of Body and Mind", "Sword of Feast and Famine", "Sword of Fire and Ice", "Sword of Light and Shadow", "Sword of Sinew and Steel", "Sword of Truth and Justice", "Sword of Vengeance", "Sword of War and Peace", "Sword of the Animist", "Sword of the Paruns", "The Reaver Cleaver", "Thornbite Staff", "Umbral Mantle", "Umezawa's Jitte", "Whispersilk Cloak"}
R3 measured deck-legal Complete subset = {"Bone Saw", "Kite Shield", "Paradise Mantle", "Sword of Feast and Famine", "Sword of Light and Shadow", "Sword of Sinew and Steel", "Sword of Truth and Justice", "Sword of War and Peace", "The Reaver Cleaver", "Umezawa's Jitte"}

thread 'pb_dx26_attach_keyword_roster::r3_deck_legal_complete_subset_of_r1_is_pinned' (3085708) panicked at crates/engine/tests/core/pb_dx26_attach_keyword_roster.rs:253:5:
assertion `left == right` failed: R3 (deck-legal `Complete` members of the Equip-marker roster) changed. A completeness FLIP is a deliberate act: re-read the def's blocker note and say in the commit why every printed clause is now implemented (or why it no longer is).
Found:    {"Bone Saw", "Kite Shield", "Paradise Mantle", "Sword of Feast and Famine", "Sword of Light and Shadow", "Sword of Sinew and Steel", "Sword of Truth and Justice", "Sword of War and Peace", "The Reaver Cleaver", "Umezawa's Jitte"}
Expected: {"Bone Saw", "Kite Shield", "Paradise Mantle", "Sword of Body and Mind", "Sword of Feast and Famine", "Sword of Light and Shadow", "Sword of Sinew and Steel", "Sword of Truth and Justice", "Sword of War and Peace", "The Reaver Cleaver", "Umezawa's Jitte"}
  left: {"Bone Saw", "Kite Shield", "Paradise Mantle", "Sword of Feast and Famine", "Sword of Light and Shadow", "Sword of Sinew and Steel", "Sword of Truth and Justice", "Sword of War and Peace", "The Reaver Cleaver", "Umezawa's Jitte"}
 right: {"Bone Saw", "Kite Shield", "Paradise Mantle", "Sword of Body and Mind", "Sword of Feast and Famine", "Sword of Light and Shadow", "Sword of Sinew and Steel", "Sword of Truth and Justice", "Sword of War and Peace", "The Reaver Cleaver", "Umezawa's Jitte"}
R4 measured Equipment defs with no ACTIVATED attach = {"Blackblade Reforged", "Blade of the Bloodchief", "Bone Saw", "Commander's Plate", "Cryptic Coat", "Empyrial Plate", "Glimmer Lens", "Illusionist's Bracers", "Kite Shield", "Lizard Blades", "Mask of Memory", "Paradise Mantle", "Quietus Spike", "Sting, the Glinting Dagger", "Sword of Body and Mind", "Sword of Feast and Famine", "Sword of Light and Shadow", "Sword of Sinew and Steel", "Sword of Truth and Justice", "Sword of War and Peace", "Sword of the Animist", "Sword of the Paruns", "The Reaver Cleaver", "Umbral Mantle", "Umezawa's Jitte"}
R4 measured Equipment defs with NO attach path at all = {"Blackblade Reforged", "Blade of the Bloodchief", "Bone Saw", "Commander's Plate", "Empyrial Plate", "Glimmer Lens", "Illusionist's Bracers", "Kite Shield", "Lizard Blades", "Mask of Memory", "Paradise Mantle", "Quietus Spike", "Sting, the Glinting Dagger", "Sword of Body and Mind", "Sword of Feast and Famine", "Sword of Light and Shadow", "Sword of Sinew and Steel", "Sword of Truth and Justice", "Sword of War and Peace", "Sword of the Animist", "Sword of the Paruns", "The Reaver Cleaver", "Umbral Mantle", "Umezawa's Jitte"}

thread 'pb_dx26_attach_keyword_roster::r4_every_equipment_subtyped_def_has_a_reachable_attach' (3085709) panicked at crates/engine/tests/core/pb_dx26_attach_keyword_roster.rs:296:5:
assertion `left == right` failed: R4 (inverse census, PB-DX26): an `Equipment`-subtyped def has no reachable attach path — neither an `AbilityDefinition::Activated` nor an `AbilityDefinition::Triggered` whose effect reaches `Effect::AttachEquipment` (walked recursively). Either author the printed Equip ability, or add the def here WITH A STATED REASON (a card that is an Equipment but genuinely prints no attach ability at all — e.g. one that only ever attaches via another card's effect).
Found:    {"Blackblade Reforged", "Blade of the Bloodchief", "Bone Saw", "Commander's Plate", "Empyrial Plate", "Glimmer Lens", "Illusionist's Bracers", "Kite Shield", "Lizard Blades", "Mask of Memory", "Paradise Mantle", "Quietus Spike", "Sting, the Glinting Dagger", "Sword of Body and Mind", "Sword of Feast and Famine", "Sword of Light and Shadow", "Sword of Sinew and Steel", "Sword of Truth and Justice", "Sword of War and Peace", "Sword of the Animist", "Sword of the Paruns", "The Reaver Cleaver", "Umbral Mantle", "Umezawa's Jitte"}
Expected: {}
  left: {"Blackblade Reforged", "Blade of the Bloodchief", "Bone Saw", "Commander's Plate", "Empyrial Plate", "Glimmer Lens", "Illusionist's Bracers", "Kite Shield", "Lizard Blades", "Mask of Memory", "Paradise Mantle", "Quietus Spike", "Sting, the Glinting Dagger", "Sword of Body and Mind", "Sword of Feast and Famine", "Sword of Light and Shadow", "Sword of Sinew and Steel", "Sword of Truth and Justice", "Sword of War and Peace", "Sword of the Animist", "Sword of the Paruns", "The Reaver Cleaver", "Umbral Mantle", "Umezawa's Jitte"}
 right: {}
R1 measured equip-marker roster (21) = {"Blackblade Reforged", "Blade of the Bloodchief", "Bone Saw", "Commander's Plate", "Empyrial Plate", "Glimmer Lens", "Illusionist's Bracers", "Kite Shield", "Mask of Memory", "Paradise Mantle", "Sword of Body and Mind", "Sword of Feast and Famine", "Sword of Light and Shadow", "Sword of Sinew and Steel", "Sword of Truth and Justice", "Sword of War and Peace", "Sword of the Animist", "Sword of the Paruns", "The Reaver Cleaver", "Umbral Mantle", "Umezawa's Jitte"}
test pb_dx26_attach_keyword_roster::r5_every_fortification_subtyped_def_has_a_reachable_attach ... ok
test pb_dx26_attach_keyword_roster::r2_every_equip_marker_def_has_a_reachable_equip_ability ... FAILED
test pb_dx26_attach_keyword_roster::r3_deck_legal_complete_subset_of_r1_is_pinned ... FAILED
test pb_dx26_attach_keyword_roster::r4_every_equipment_subtyped_def_has_a_reachable_attach ... FAILED
test pb_dx26_attach_keyword_roster::r1_equip_keyword_marker_roster_is_pinned ... ok

failures:

failures:
    pb_dx26_attach_keyword_roster::r2_every_equip_marker_def_has_a_reachable_equip_ability
    pb_dx26_attach_keyword_roster::r3_deck_legal_complete_subset_of_r1_is_pinned
    pb_dx26_attach_keyword_roster::r4_every_equipment_subtyped_def_has_a_reachable_attach
    pb_dx26_attach_keyword_roster::r6_effect_nesting_sites_are_pinned

test result: FAILED. 2 passed; 4 failed; 0 ignored; 0 measured; 502 filtered out; finished in 0.01s

error: test failed, to rerun pass `-p mtg-engine --test core`

```

## Revert matrix — executed, not argued

### V1 — bone_saw loses its authored equip ability entirely (the OOS-CARDS1-3 defect, restored)

**RED (as required)**

```
test pb_dx26_equip_surface::t1_bone_saw_equip_ability_exists_and_offers_exactly_its_own_controllers_creature ... FAILED
test pb_dx26_equip_surface::t9_guardian_project_draws_on_a_nontoken_etb_and_not_on_a_token_one ... ok
test pb_dx26_equip_surface::t7_darksteel_garrison_fortify_offers_only_a_land_its_controller_owns ... ok
test pb_dx26_equip_surface::t4_bone_saw_equip_rejects_an_opponents_creature ... FAILED
test pb_dx26_equip_surface::t5_bone_saw_zero_target_activation_is_rejected ... FAILED
test pb_dx26_equip_surface::t2_bone_saw_equip_attaches_and_the_printed_static_applies ... FAILED
test pb_dx26_equip_surface::t3_umezawas_jitte_equip_is_appended_and_does_not_renumber_the_modal_ability ... ok
test pb_dx26_equip_surface::t8_darksteel_garrison_fortify_attaches_and_grants_indestructible ... ok
test pb_dx26_equip_surface::t6_equip_keyword_marker_is_retained_alongside_the_authored_ability ... FAILED
thread 'pb_dx26_equip_surface::t1_bone_saw_equip_ability_exists_and_offers_exactly_its_own_controllers_creature' (3178059) panicked at crates/engine/tests/primitives/pb_dx26_equip_surface.rs:222:9:
thread 'pb_dx26_equip_surface::t4_bone_saw_equip_rejects_an_opponents_creature' (3178062) panicked at crates/engine/tests/primitives/pb_dx26_equip_surface.rs:380:51:
thread 'pb_dx26_equip_surface::t5_bone_saw_zero_target_activation_is_rejected' (3178063) panicked at crates/engine/tests/primitives/pb_dx26_equip_surface.rs:404:51:
thread 'pb_dx26_equip_surface::t2_bone_saw_equip_attaches_and_the_printed_static_applies' (3178060) panicked at crates/engine/tests/primitives/pb_dx26_equip_surface.rs:273:51:
thread 'pb_dx26_equip_surface::t6_equip_keyword_marker_is_retained_alongside_the_authored_ability' (3178064) panicked at crates/engine/tests/primitives/pb_dx26_equip_surface.rs:436:5:
test result: FAILED. 4 passed; 5 failed; 0 ignored; 0 measured; 1142 filtered out; finished in 0.05s
```

### V1b — same reversion, roster gate view (R2 must list Bone Saw again)

**RED (as required)**

```
test pb_dx26_attach_keyword_roster::r6_effect_nesting_sites_are_pinned ... ok
test pb_dx26_attach_keyword_roster::r1_equip_keyword_marker_roster_is_pinned ... ok
test pb_dx26_attach_keyword_roster::r3_deck_legal_complete_subset_of_r1_is_pinned ... ok
test pb_dx26_attach_keyword_roster::r2_every_equip_marker_def_has_a_reachable_equip_ability ... FAILED
test pb_dx26_attach_keyword_roster::r5_every_fortification_subtyped_def_has_a_reachable_attach ... ok
test pb_dx26_attach_keyword_roster::r4_every_equipment_subtyped_def_has_a_reachable_attach ... FAILED
thread 'pb_dx26_attach_keyword_roster::r2_every_equip_marker_def_has_a_reachable_equip_ability' (3178502) panicked at crates/engine/tests/core/pb_dx26_attach_keyword_roster.rs:225:5:
thread 'pb_dx26_attach_keyword_roster::r4_every_equipment_subtyped_def_has_a_reachable_attach' (3178504) panicked at crates/engine/tests/core/pb_dx26_attach_keyword_roster.rs:336:5:
test result: FAILED. 4 passed; 2 failed; 0 ignored; 0 measured; 502 filtered out; finished in 0.01s
```

### V1c — same reversion, re-pinned R1 must drop to 37

**RED (as required)**

```
test cards1_equip_target_roster::r3_walk_is_not_vacuous ... FAILED
test cards1_equip_target_roster::r1_equip_activated_attach_equipment_roster_is_pinned ... FAILED
test cards1_equip_target_roster::r2_every_roster_member_has_exactly_the_expected_target_requirement ... ok
thread 'cards1_equip_target_roster::r3_walk_is_not_vacuous' (3178520) panicked at crates/engine/tests/core/cards1_equip_target_roster.rs:190:5:
thread 'cards1_equip_target_roster::r1_equip_activated_attach_equipment_roster_is_pinned' (3178518) panicked at crates/engine/tests/core/cards1_equip_target_roster.rs:170:5:
test result: FAILED. 1 passed; 2 failed; 0 ignored; 0 measured; 505 filtered out; finished in 0.01s
```

### V2 — darksteel_garrison's fortify target removed (OOS-CARDS1-1's original shape)

**RED (as required)**

```
test pb_dx26_equip_surface::t7_darksteel_garrison_fortify_offers_only_a_land_its_controller_owns ... FAILED
thread 'pb_dx26_equip_surface::t7_darksteel_garrison_fortify_offers_only_a_land_its_controller_owns' (3179758) panicked at crates/engine/tests/primitives/pb_dx26_equip_surface.rs:524:5:
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1150 filtered out; finished in 0.03s
```

### V2b — same reversion, the strengthened t7b requirement pin

**RED (as required)**

```
test cards1_equip_target_repair::t7b_fortify_and_reconfigure_rosters_pinned_and_unperturbed ... FAILED
thread 'cards1_equip_target_repair::t7b_fortify_and_reconfigure_rosters_pinned_and_unperturbed' (3179770) panicked at crates/engine/tests/primitives/cards1_equip_target_repair.rs:760:21:
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1150 filtered out; finished in 0.01s
```

### V2c — darksteel_garrison given the EQUIP repair's TargetCreatureWithFilter (the copy-paste mistake OOS-CARDS1-1 warns about)

**RED (as required)**

```
test pb_dx26_equip_surface::t7_darksteel_garrison_fortify_offers_only_a_land_its_controller_owns ... FAILED
thread 'pb_dx26_equip_surface::t7_darksteel_garrison_fortify_offers_only_a_land_its_controller_owns' (3180971) panicked at crates/engine/tests/primitives/pb_dx26_equip_surface.rs:534:5:
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1150 filtered out; finished in 0.03s
```

### V3 — guardian_project's is_nontoken flipped back to false (OOS-DX3b-1's (a) half, un-applied)

**RED (as required)**

```
test pb_dx26_equip_surface::t9_guardian_project_draws_on_a_nontoken_etb_and_not_on_a_token_one ... FAILED
thread 'pb_dx26_equip_surface::t9_guardian_project_draws_on_a_nontoken_etb_and_not_on_a_token_one' (3182215) panicked at crates/engine/tests/primitives/pb_dx26_equip_surface.rs:780:5:
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1150 filtered out; finished in 0.03s
```

### V4 — bone_saw's CR 702.6a requirement weakened to a bare TargetCreature (drops 'you control')

**RED (as required)**

```
test cards1_equip_target_roster::r2_every_roster_member_has_exactly_the_expected_target_requirement ... FAILED
thread 'cards1_equip_target_roster::r2_every_roster_member_has_exactly_the_expected_target_requirement' (3183507) panicked at crates/engine/tests/core/cards1_equip_target_roster.rs:276:5:
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 507 filtered out; finished in 0.01s
```

### V4b — same reversion, behavioural view: an opponent's creature becomes equippable

***** STILL GREEN — NOT DISCRIMINATING *****

```
test pb_dx26_equip_surface::t4_bone_saw_equip_rejects_an_opponents_creature ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1150 filtered out; finished in 0.03s
```

### V5 — R6's Effect-nesting-site count mis-pinned (stands in for a new Box<Effect> field being added)

**RED (as required)**

```
test pb_dx26_attach_keyword_roster::r6_effect_nesting_sites_are_pinned ... FAILED
thread 'pb_dx26_attach_keyword_roster::r6_effect_nesting_sites_are_pinned' (3185070) panicked at crates/engine/tests/core/pb_dx26_attach_keyword_roster.rs:483:5:
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 507 filtered out; finished in 0.00s
```

### V6a — bone_saw's attach nested inside an Effect::Sequence, WITH PB-DX26's recursive walk in place (must stay GREEN — this is the fix working)

***** STILL GREEN — NOT DISCRIMINATING *****

```
test cards1_equip_target_roster::r1_equip_activated_attach_equipment_roster_is_pinned ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 507 filtered out; finished in 0.01s
```

### V6b — same nesting, but the walk reverted to the pre-PB-DX26 flat matches! (the seed-rerank §2.7 hazard: Bone Saw silently drops out of the exact pin)

**RED (as required)**

```
test cards1_equip_target_roster::r1_equip_activated_attach_equipment_roster_is_pinned ... FAILED
thread 'cards1_equip_target_roster::r1_equip_activated_attach_equipment_roster_is_pinned' (3186555) panicked at crates/engine/tests/core/cards1_equip_target_roster.rs:170:5:
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 507 filtered out; finished in 0.01s
```

### V7 — Quietus Spike stops being Inert while still having no attach path (the residual's stated reason no longer holds)

**RED (as required)**

```
test pb_dx26_attach_keyword_roster::r4_every_equipment_subtyped_def_has_a_reachable_attach ... FAILED
thread 'pb_dx26_attach_keyword_roster::r4_every_equipment_subtyped_def_has_a_reachable_attach' (3187735) panicked at crates/engine/tests/core/pb_dx26_attach_keyword_roster.rs:359:9:
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 507 filtered out; finished in 0.01s
```

### V8 — Umezawa's Jitte's equip ability inserted BEFORE the PB-EF7 modal ability (renumbers ability_index 0 -> 1)

**RED (as required)**

```
test pb_dx26_equip_surface::t3_umezawas_jitte_equip_is_appended_and_does_not_renumber_the_modal_ability ... FAILED
thread 'pb_dx26_equip_surface::t3_umezawas_jitte_equip_is_appended_and_does_not_renumber_the_modal_ability' (3188846) panicked at crates/engine/tests/primitives/pb_dx26_equip_surface.rs:339:5:
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1150 filtered out; finished in 0.03s
```

### V4c — the same reversion, offer-side view (`t1`)

**RED (as required).** With `bone_saw`'s requirement weakened to a bare
`TargetCreature`, the offer half fails:

```
test pb_dx26_equip_surface::t1_bone_saw_equip_ability_exists_and_offers_exactly_its_own_controllers_creature ... FAILED
panicked at pb_dx26_equip_surface.rs:252:
CR 702.6a: an OPPONENT's creature must NOT be offered — the requirement carries
`controller: TargetController::You`. Slot: [Object(ObjectId(2)), Object(ObjectId(3))]
```

## 3. The one row that did NOT discriminate, stated rather than glossed

**V4b is honestly UNDISCRIMINATED.** `t4` asserts that naming an opponent's
creature as an equip target is *rejected*, and it stays GREEN with the CR 702.6a
requirement weakened to a bare `TargetCreature`. That is not a flaw in the fix and
not a false claim by the test — it is `OOS-DX20-7`: `rules/abilities.rs` carries a
legacy `Effect::AttachEquipment` special-case that validates a **volunteered**
target's creature-ness *and* controller, and it is precisely because that guard
never *required* a target that `OOS-M11-10(equip)` was a silent fizzle rather than
a visible error. So the validation-side rejection has two independent providers and
`t4` cannot tell which one answered.

What this means for reading the evidence:

* The **"you control" clause is proven** by `t1` (offer side, V4c above) and by
  `core::cards1_equip_target_roster::r2` (shape side, V4 above). Both go red.
* `t4` is kept as a behavioural pin of the CR rule at the command boundary — it
  would catch a regression that removed *both* providers — but it must not be read
  as evidence that the requirement is what rejects. Its doc comment says so.

This is the same shape as PB-DX25c's V3/V7 rows: an assertion shadowed by a
redundant downstream check is still worth having and is not worth overclaiming.
