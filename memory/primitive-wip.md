# Primitive WIP: PB-S — GrantActivatedAbility

batch: PB-S
title: GrantActivatedAbility — static effect that grants an activated ability to filtered permanents
cards_affected: ~8+ (Cryptolith Rite, Citanul Hierophants, Chromatic Lantern, Paradise Mantle, Umbral Mantle, Enduring Vitality, Song of Freyalise, Marvin Murderous Mimic partial)
started: 2026-04-11
phase: plan
plan_file: memory/primitives/pb-plan-S.md

## Deferred from Prior PBs
- Heritage Druid `TapNCreatures` cost — larger cost-framework change, separate PB
- Marvin "grant all other activated abilities" reflection pattern — out of scope for PB-S (static grants one specific ability, not arbitrary ability copy)
- Tier 1 authoring blockers (pump on all creatures, dynamic P/T, ExileSelf) — **PB-X micro-PB**

## Session Goal
PLAN PHASE ONLY this session. Produce `memory/primitives/pb-plan-S.md` describing:
- CR citations (112 activated abilities, 611 continuous effects, static ability grants via layer 6)
- Engine architecture: new `Effect::GrantActivatedAbility { filter, granted_ability }` or a static-ability variant
- How the granted ability shows up in `legal_actions` (activation must be discoverable by the LegalActionProvider for TUI/simulator to use it)
- Layer system integration (layer 6 — text-changing / ability-granting, same layer as Flying grants)
- How the granted activation cost is paid from the target permanent, not the grantor
- Card def DSL shape for the 8+ cards
- Test plan (5-8 unit tests)

## Step Checklist (PLAN)
- [x] 1. Research CR 112 (activated abilities), 611 (continuous effects), layer 6 ability-granting
- [x] 2. Study existing Flying-grant pattern (e.g. Levitation, Concordant Crossroads for haste grants)
- [x] 3. Study how Cryptolith Rite, Chromatic Lantern are currently TODO'd in defs
- [x] 4. Write plan file memory/primitives/pb-plan-S.md
- [ ] 5. Do NOT implement this session — stop at plan
