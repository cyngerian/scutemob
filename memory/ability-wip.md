# Ability WIP: PB-2 Conditional ETB Tapped

ability: Conditional ETB Tapped
cr: 614.1c (replacement effects — enters tapped unless)
priority: W6-PB-2
started: 2026-03-14
phase: implement
plan_file: (inline — see primitive-card-plan.md PB-2)

## Step Checklist
- [x] 1. New Condition variants (8 variants: Or, ControlLandWithSubtypes, ControlAtMostNOtherLands, HaveTwoOrMoreOpponents, CanRevealFromHandWithSubtype, ControlBasicLandsAtLeast, ControlAtLeastNOtherLands, ControlAtLeastNOtherLandsWithSubtype)
- [x] 2. unless_condition: Option<Condition> on AbilityDefinition::Replacement (avoids circular dep)
- [x] 3. check_condition arms + replacement.rs unless_condition check
- [x] 4. Hash discriminants 19-26 + exhaustive match updates (116 card defs sed'd)
- [x] 5. Unit tests — 8 tests covering all condition patterns (77 total in replacement_effects.rs)
- [x] 6. Fix 56 card defs — all unless_condition set (18 check/castle, 3 fast, 8 slow, 5 battle, 10 bond, 8 reveal, 2 subtype-count, 2 special)
- [x] 7. Build verification — 1990 tests passing, clippy clean, workspace builds

## Design

### New Condition variants:
- `ControlLandWithSubtypes(Vec<SubType>)` — check-lands + castles (any of listed subtypes)
- `ControlAtMostNOtherLands(u32)` — fast-lands (≤ N other lands → enters untapped)
- `HaveTwoOrMoreOpponents` — bond-lands (≥ 2 opponents → enters untapped)
- `CanRevealFromHandWithSubtype(SubType)` — reveal-lands (deterministic auto-reveal)
- `ControlBasicLandsAtLeast(u32)` — battle-lands (≥ N basics → enters untapped)
- `ControlAtLeastNOtherLands(u32)` — slow-lands (≥ N other lands → enters untapped)

### ReplacementModification:
- `EntersTappedUnless(Condition)` — if condition met, enter untapped; if not, enter tapped

### Evaluation:
- Construct minimal EffectContext in emit_etb_modification (controller + source)
- Reuse existing check_condition infrastructure

### Card sub-patterns (56 total):
- Check-lands (12): clifftop_retreat, dragonskull_summit, drowned_catacomb, glacial_fortress, etc.
- Fast-lands (6): blooming_marsh, concealed_courtyard, darkslick_shores, etc.
- Bond-lands (6-10): bountiful_promenade, luxury_suite, morphic_pool, etc.
- Reveal-lands (6): choked_estuary, foreboding_ruins, frostboil_snarl, etc.
- Battle-lands (5): canopy_vista, cinder_glade, prairie_stream, smoldering_marsh, sunken_hollow
- Slow-lands (6): deathcap_glade, dreamroot_cascade, haunted_ridge, rockfall_vale, shipwreck_marsh, stormcarved_coast
- Misc (remaining): castle_*, arena_of_glory, flamekin_village, mystic_sanctuary, etc.
