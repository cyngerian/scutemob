# Ability WIP: PB-3 Shockland ETB (pay-life-or-tapped)

ability: Shockland ETB (pay-life-or-tapped)
cr: 614.1c (replacement effects — enters tapped unless pay life)
priority: W6-PB-3
started: 2026-03-14
phase: complete
plan_file: (inline — see primitive-card-plan.md PB-3)

## Step Checklist
- [x] 1. Add ReplacementModification::EntersTappedUnlessPayLife(u32) variant
- [x] 2. Handle in apply_self_etb_from_definition (replacement.rs) — deterministic: enters tapped
- [x] 3. Hash discriminant 8 in hash.rs
- [x] 4. Unit tests — 3 tests (single shockland, all 10, variant check)
- [x] 5. Fix 10 card defs — add replacement + dual mana abilities
- [x] 6. Build verification — 1993 tests passing, clippy clean, workspace builds

## Design

### Engine change:
- `ReplacementModification::EntersTappedUnlessPayLife(u32)` — "As this enters, you may pay N life. If you don't, it enters tapped."
- Deterministic fallback (pre-M10): always enters tapped (conservative, prevents free mana)
- Combined match arm with `EntersTapped` in `emit_etb_modification` — same behavior, distinct variant for M10 wiring
- Interactive choice deferred to M10 (Command::PayLifeForETB or similar)

### Cards (10):
- blood_crypt (B/R), breeding_pool (G/U), godless_shrine (W/B), hallowed_fountain (W/U),
  overgrown_tomb (B/G), sacred_foundry (R/W), steam_vents (U/R), stomping_ground (R/G),
  temple_garden (G/W), watery_grave (U/B)
