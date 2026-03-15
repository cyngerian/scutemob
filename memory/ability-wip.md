# Ability WIP: PB-8 Cost Reduction Statics

ability: Cost Reduction Statics
cr: 601.2f (total cost determination), 601.2e (cost increases/reductions)
priority: W6-PB-8
started: 2026-03-14
phase: complete
plan_file: (inline — see docs/primitive-card-plan.md PB-8)

## Step Checklist
- [x] 1. Add SpellCostModifier + SpellCostFilter + SelfCostReduction types to card_definition.rs
- [x] 2. Add apply_spell_cost_modifiers() + apply_self_cost_reduction() to casting.rs
- [x] 3. Hash discriminants — N/A (types on CardDefinition, not game state)
- [x] 4. Unit tests (8 tests in spell_cost_modification.rs)
- [x] 5. Fix 5 permanent-modifier cards (Thalia, Warchief, Jhoira's, Danitha, Ur-Dragon)
- [x] 6. Fix 5 self-reduction cards (Blasphemous Act, Ghalta, Emrakul, Scion of Draco, Earthquake Dragon)
- [x] 7. Build verification — workspace builds, clippy clean, 2028 tests, 0 failures

## Design

### Two mechanisms:

**A) SpellCostModifier** — permanent on battlefield (or command zone for Eminence) modifies costs of spells being cast:
- `SpellCostModifier { change: i32, filter: SpellCostFilter, scope: CostModifierScope, eminence: bool }`
- `SpellCostFilter`: NonCreature, HasSubtype(SubType), Historic, HasCardType(CardType), AuraOrEquipment
- `CostModifierScope`: AllPlayers (Thalia), Controller (Warchief)
- Pipeline position: after commander tax + kicker, before affinity/undaunted

**B) SelfCostReduction** — spell itself is cheaper based on game state at cast time:
- `SelfCostReduction::PerPermanent { per, filter, controller }` — Blasphemous Act
- `SelfCostReduction::TotalPowerOfCreatures` — Ghalta
- `SelfCostReduction::CardTypesInGraveyard` — Emrakul
- `SelfCostReduction::BasicLandTypes { per }` — Scion of Draco (Domain)
- `SelfCostReduction::TotalManaValue { filter }` — Earthquake Dragon
- Pipeline position: same as (A), right after it

### Cards (10):
**Permanent modifiers (5):** thalia_guardian_of_thraben (+1 noncreature), goblin_warchief (-1 Goblin + haste grant),
  jhoiras_familiar (-1 Historic), danitha_capashen_paragon (-1 Aura/Equipment),
  the_ur_dragon (-1 Dragon, Eminence)
**Self-reduction (5):** blasphemous_act (per creature), ghalta_primal_hunger (total power),
  emrakul_the_promised_end (card types in GY), scion_of_draco (basic land types),
  earthquake_dragon (total MV of Dragons)

### Additional changes:
- 129+ card def files: added `spell_cost_modifiers: vec![], self_cost_reduction: None,` to explicit struct constructions
- 4 DFC card def files: fixed field placement after back_face: Some(...)
- Test files (15+): added fields to explicit CardDefinition constructions
- Goblin Warchief: also added haste grant static ability (was previously TODO)
