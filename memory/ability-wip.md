# Ability WIP: Morph Mini-Milestone

ability: Morph (+ Megamorph, Disguise, Manifest, Cloak)
cr: 702.37 (Morph), 702.37f (Megamorph), 702.162 (Disguise), 701.40 (Manifest), 701.58 (Cloak)
priority: P3 (Morph) + P4 (Megamorph, Disguise, Manifest, Cloak)
started: 2026-03-08
phase: closed
plan_file: memory/abilities/ability-plan-morph.md
review_file: memory/abilities/ability-review-morph.md

## Step Checklist
- [x] 1. Plan — CR research, face-down object model design, implementation plan
- [x] 2. Implement — face-down model, CastFaceDown command, TurnFaceUp command, layer overrides, tests (crates/engine/tests/morph.rs — 14 tests passing)
- [x] 3. Review — 1 HIGH, 3 MEDIUM, 2 LOW findings (see review file)
- [x] 4. Fix — apply HIGH/MEDIUM findings (abilities.rs:5590-5599, stack.rs:1376+, resolution.rs:6970+, hash.rs:2190+, sba.rs:434+, state/mod.rs:600+, legal_actions.rs:514+, morph.rs:test_face_down_dies_reveal)
- [x] 5. Cards — Exalted Angel, Birchlore Rangers, Akroma Angel of Fury (3 Morph card defs)
- [x] 6. Scripts — 197 (cast face-down + turn face-up, PASS), 198 (face-down creature dies revealed, PASS); script 199 skipped (no Megamorph card in registry)
- [x] 7. Close — coverage doc, CLAUDE.md, MEMORY.md, workstream-state.md, workstream-coordination.md updated

## Review Summary (2026-03-08)
- **F1 (HIGH)**: Manifest/Cloak ETB suppression gap — `collect_triggers_for_event` reads raw characteristics, fires ETB on face-down entry
- **F2 (MEDIUM)**: TurnFaceUpTrigger SOK missing ability_index — fires wrong ability for cards with multiple WhenTurnedFaceUp triggers
- **F3 (MEDIUM)**: FaceDownRevealed event defined but never emitted — CR 708.9 zone-change reveal
- **F4 (MEDIUM)**: LegalActionProvider missing TurnFaceUp and morph-cast actions
- **F5 (LOW)**: `.unwrap()` in engine.rs:1538
- **F6 (LOW)**: `face_down_kind` CastSpell field ignored

## Discriminant chain start
KW: 153 (next after 152)
AbilDef: 62 (next after 61)
SOK: 63 (next after 62)
