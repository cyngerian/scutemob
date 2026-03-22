# Primitive WIP: PB-22 S7 -- Adventure + Dual-Zone Search

batch: PB-22
session: S7
title: Adventure (CR 715) + dual-zone search
cards_affected: ~8 (Monster Manual, Lozhan + ~3 adventure cards + Finale of Devastation pattern)
started: 2026-03-21
phase: closed
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
  - ALL 9 PASSING

- [x] 5. Workspace build verification
  - cargo test --all: 2281 passing, 0 failing
  - cargo clippy -- -D warnings: clean
  - cargo build --workspace: clean
  - cargo fmt --check: clean

## Review
findings: 4 (HIGH: 1, MEDIUM: 3, LOW: 0)
verdict: fixed
review_file: memory/primitives/pb-review-22-s7.md

Fixes applied (2026-03-21):
- [x] HIGH-3: monster_manual.rs — replaced SearchLibrary adventure face with Effect::MillCards(5) + Effect::MoveZone to hand; target: TargetCardInYourGraveyard(Creature); oracle_text corrected
- [x] MEDIUM-1: copy.rs:207 — was_cast_as_adventure: original.was_cast_as_adventure (CR 715.3c); cascade/discover sites remain false (correct — new casts, not copies)
- [x] MEDIUM-2: legal_actions.rs — added TODO(W2) block in StubProvider doc comment documenting both Adventure casting gaps (cast as adventure from hand, cast creature from adventure exile)
- [x] MEDIUM-4: bonecrusher_giant.rs — updated TODO to document 3 gaps: (1) WhenBecomesTargetByOpponent is WRONG (should be WhenBecomesTargetBySpell for any spell); (2) EachOpponent is WRONG (needs EffectTarget::TriggeringPlayer); (3) prevention removal omitted
