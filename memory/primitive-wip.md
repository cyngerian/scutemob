# Primitive WIP: PB-22 S7 -- Adventure + Dual-Zone Search

batch: PB-22
session: S7
title: Adventure (CR 715) + dual-zone search
cards_affected: ~8 (Monster Manual, Lozhan + ~3 adventure cards + Finale of Devastation pattern)
started: 2026-03-21
phase: implement
plan_file: memory/primitives/pb-plan-22-s7.md

## Deferred from Prior PBs
- Adventure (cast from exile) — PB-13m
- Dual-zone search (library OR graveyard) — Finale of Devastation pattern

## Step Checklist
- [x] 1. Engine changes: AltCostKind::Adventure, exile-on-resolution, cast creature from exile
  - Added `AltCostKind::Adventure` (disc 27) to types.rs
  - Added `adventure_face: Option<CardFace>` to CardDefinition
  - Added `was_cast_as_adventure: bool` to StackObject
  - Added `adventure_exiled_by: Option<PlayerId>` to GameObject
  - casting.rs: cast_with_adventure binding, zone validation, type override (CR 715.3a), cost from adventure_face
  - resolution.rs: is_permanent=false override, adventure face effect selection, exile-on-resolution + adventure_exiled_by set (CR 715.3d)
  - resolution.rs: comment at fizzle/counter paths (CR 715.3d: exile only on resolution)
  - hash.rs: AltCostKind::Adventure => 27, was_cast_as_adventure on StackObject, adventure_exiled_by on GameObject

- [x] 2. Engine changes: dual-zone search (extend SearchLibrary or new Effect)
  - Added `also_search_graveyard: bool` to Effect::SearchLibrary (card_definition.rs)
  - Updated effects/mod.rs: candidate collection includes graveyard when also_search_graveyard=true
  - Updated hash.rs: also_search_graveyard in SearchLibrary arm
  - Updated dungeon.rs: also_search_graveyard: false in The Undercity room
  - Updated all 26 card defs with SearchLibrary (also_search_graveyard: false by default)

- [x] 3. Card definition fixes (Monster Manual, etc.)
  - monster_manual.rs: added adventure_face with Zoological Study (SearchLibrary creature, then shuffle)
  - finale_of_devastation.rs: also_search_graveyard: true (dual-zone search implemented), updated TODOs
  - lozhan_dragons_legacy.rs: updated TODO to note Adventure framework now exists
  - bonecrusher_giant.rs: NEW — Bonecrusher Giant // Stomp with adventure_face
  - lovestruck_beast.rs: NEW — Lovestruck Beast // Heart's Desire with adventure_face
  - adventure_face: None added to all 136 card defs that needed it

- [x] 4. Unit tests (Adventure: 5, dual-zone: 3)
  - adventure_tests.rs: 9 tests (6 Adventure + 3 dual-zone)
    - test_adventure_cast_adventure_half_from_hand (CR 715.3a, 715.3b)
    - test_adventure_exile_on_resolution (CR 715.3d)
    - test_adventure_cast_creature_from_exile (CR 715.3d)
    - test_adventure_countered_goes_to_graveyard (CR 715.3d)
    - test_adventure_cannot_recast_as_adventure_from_exile (CR 715.3d)
    - test_adventure_normal_characteristics_in_hand (CR 715.4)
    - test_search_library_only (CR 701.23)
    - test_search_library_and_graveyard (CR 701.23)
    - test_search_graveyard_still_shuffles_library (CR 701.23)
  - ALL 9 PASSING

- [x] 5. Workspace build verification
  - cargo test --all: 2281 passing, 0 failing
  - cargo clippy -- -D warnings: clean
  - cargo build --workspace: clean
  - cargo fmt --check: clean
