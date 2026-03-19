# Primitive WIP: PB-17 -- Library Search Filters (REVIEW-ONLY)

batch: PB-17
title: Library Search Filters
cards_affected: 27 (card defs using SearchLibrary; plan said 74 but most are unwritten)
mode: review-only
started: 2026-03-19
phase: fix
plan_file: n/a (retroactive review -- no plan needed)

## Review
findings: 9 (HIGH: 4, MEDIUM: 4, LOW: 1)
verdict: needs-fix
review_file: memory/primitives/pb-review-17.md

## Fix Checklist

### HIGH (wrong game state)
- [x] 1. Vampiric Tutor: shuffle undoes top-of-library placement — added shuffle_before_placing: true, removed separate Shuffle step
- [x] 2. Worldly Tutor: same shuffle-order bug — same fix as #1
- [x] 3. Assassin's Trophy: target filter wrong — changed to TargetController::Opponent, removed non_land: true
- [x] 4. Prismatic Vista: enters tapped, oracle says untapped — changed tapped: true to tapped: false

### MEDIUM
- [x] 5. Boseiju: replaced basic: true with has_subtypes vec of 5 basic land type subtypes (CR 305.8)
- [x] 6. Crop Rotation: wrapped SearchLibrary in Sequence with Shuffle step after
- [x] 7. Kodama's Reach: changed types() to types_sub() with "Arcane" subtype
- [x] 8. CR citation fix: all SearchLibrary contexts updated 701.19 → 701.23 (Regenerate references untouched)

### DEFERRED
- Urza's Saga mana value vs mana cost (DSL gap — exact_mana_cost filter needed)
- All LOW items (known deferred TODOs)
