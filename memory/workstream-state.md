# Workstream State

> Coordination file for parallel sessions. Read by `/start-session`, claimed by
> `/start-work`, released by `/end-session`. This file is the source of truth for
> which workstreams are actively being worked on.
>
> **Protocol**: Read before starting. Claim before coding. Release when done.

## Active Claims

| Workstream | Task | Status | Claimed | Notes |
|------------|------|--------|---------|-------|
| W1: Abilities | — | available | — | B16 complete (Dungeon + Ring); all abilities done |
| W2: TUI & Simulator | — | available | — | Phase 1 done; 6 UX fixes done; hardening pending |
| W3: LOW Remediation | LOW remediation — T2/T3 items | available | — | Phase 0 complete; T2 done; T3 ManaPool pending |
| W4: M10 Networking | — | not-started | — | After W1 completes |
| W5: Card Authoring | — | **RETIRED** | — | Replaced by W6. See `docs/primitive-card-plan.md` |
| W6: Primitive + Card Authoring | PB-9.5: Architecture cleanup | ACTIVE | 2026-03-14 | Trigger flush discipline + test CardDefinition defaults migration |

**Status values**: `available` (free to claim), `ACTIVE` (session working on it),
`paused` (partially done, session ended mid-task), `not-started` (blocked/deferred),
`RETIRED` (replaced by another workstream)

## Last Handoff

**Date**: 2026-03-14
**Workstream**: W6: Primitive + Card Authoring
**Task**: PB-9 — Hybrid mana, Phyrexian mana, X costs

**Completed**:
- 3 new enums: HybridMana (ColorColor/GenericColor), PhyrexianMana (Single/Hybrid), HybridManaPayment (Color/Generic)
- 3 new ManaCost fields: hybrid, phyrexian, x_count (all #[serde(default)])
- mana_value() updated for CR 202.3e-g (hybrid largest component, phyrexian=1, X=0 off stack)
- flatten_hybrid_phyrexian() helper resolves payment choices into flat ManaCost + life cost
- CastSpell gains hybrid_choices + phyrexian_life_payments fields
- Color identity updated in commander.rs + casting.rs for hybrid/phyrexian (CR 903.4)
- Hash updates for new types in hash.rs
- 16 new tests in mana_costs.rs (MV, payment flatten, color identity)
- 12 card def fixes: 5 hybrid creatures, 2 hybrid mutate costs, 4 filter land hybrid activation costs, 1 hybrid Phyrexian (Ajani)
- 3 X-cost card fixes: mockingbird, cut_ribbons, treasure_vault
- 3 phyrexian card TODO updates (skrelv, tekuthal, drivnod — phyrexian cost now representable)
- ~130 test/harness/simulator files: added hybrid_choices + phyrexian_life_payments to CastSpell
- Commit c43a1fa; 2044 tests, 0 clippy warnings

**Next**:
1. Follow execution order in `docs/primitive-card-plan.md` — PB-10 through PB-21
2. **PB-10**: Return from zone effects (8 cards) — TargetCardInGraveyard targeting
3. Many PB-7 cards remain blocked on deeper gaps (dynamic LayerModification for CDA */* creatures)
4. ~23 PB-5 + ~20 PB-6 cards remain blocked on future primitives

**Hazards**: CLAUDE.md has uncommitted changes (test count update). No conflicts.

**Commit prefix used**: `W6-prim:`

## Handoff History

### 2026-03-14 — W6: PB-9 hybrid/phyrexian/X mana (19 cards)
- 3 enums + 3 ManaCost fields + flatten helper + 12 card fixes + 16 tests; commit c43a1fa; 2044 tests

### 2026-03-14 — W6: PB-8 cost reduction statics (10 cards)
- SpellCostModifier + SelfCostReduction + 10 card fixes + 8 tests; commit c1edb48; 2028 tests

### 2026-03-14 — W6: PB-7 count-based scaling (29 cards)
- 3 EffectAmount variants + AddManaScaled + 5 card fixes + 8 tests; commit 399c8da; 2020 tests

### 2026-03-14 — W6: PB-6 static grant with controller filter (30 cards)
- 3 EffectFilter variants + 10 card fixes + 6 tests; commit f722014; 2012 tests

### 2026-03-14 — W6: PB-5 targeted activated/triggered abilities (32 cards)
- targets field on Activated/Triggered + validation + 9 card fixes + 9 tests; commit 9ca054a; 2006 tests
