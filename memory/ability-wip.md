# Ability WIP: PB-9 Hybrid/Phyrexian/X Mana Costs

ability: Hybrid, Phyrexian, and X Mana Costs
cr: 202.2d (hybrid), 202.2e (Phyrexian), 107.3 (X)
priority: W6-PB-9
started: 2026-03-14
phase: complete
plan_file: memory/abilities/ability-plan-pb9-mana-costs.md

## Step Checklist
- [x] 1. Add HybridMana, PhyrexianMana enums + hybrid/phyrexian/x_count fields on ManaCost
- [x] 2. Update mana_value() for hybrid, phyrexian, X
- [x] 3. Update casting.rs: flatten_hybrid_phyrexian() helper, X cost x_count support
- [x] 4. Add hybrid_choices + phyrexian_life_payments to CastSpell command
- [x] 5. Update color identity (commander.rs, casting.rs) for hybrid/phyrexian
- [x] 6. Hash updates (hash.rs) for new types
- [x] 7. Export new types (lib.rs, state/mod.rs, helpers.rs)
- [x] 8. Update all CastSpell construction sites (~200+ across tests/simulator/harness)
- [x] 9. 16 unit tests (mana_costs.rs) — MV, payment, color identity
- [x] 10. Fix 8 hybrid card defs (kitchen_finks, boggart_ram_gang, blade_historian, connive, revitalizing_repast, brokkos, nethroi + 4 filter lands)
- [x] 11. Fix 1 hybrid Phyrexian card def (ajani_sleeper_agent)
- [x] 12. Fix 3 X-cost card defs (mockingbird, cut_ribbons, treasure_vault)
- [x] 13. Updated TODO comments on 3 phyrexian cards (skrelv, tekuthal, drivnod)
- [x] 14. Build verification — workspace builds, clippy clean, 2044 tests, 0 failures
