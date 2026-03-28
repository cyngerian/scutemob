---
name: Architecture Review Findings (2026-03-26)
description: External architecture review identified 7 concerns — 3 already handled, 4 tracked in roadmap as new deliverables
type: project
---

External review of engine architecture (`docs/scutemob-architecture-review.md`) produced 7 concerns.
Counter-assessment checked each against actual codebase.

**Already handled:**
- Replacement effect ordering — `Command::OrderReplacements` + `pending_zone_changes` already implemented
- Resolution choices — already planned for M10 (`Command::SelectLibraryCard` deferred)
- Trigger taxonomy — current enum approach works at ~2,000 card scale; revisit at 5,000+

**Tracked in roadmap (new deliverables added 2026-03-26):**
- **M10 (first)**: Cost modifiers through layer system — `spell_cost_modifiers` on `CardDefinition` bypasses layers, so Humility/Dress Down don't suppress Thalia-style taxes. Invariant violation, not just edge case. Fix before more cards authored against broken assumption.
- **M10**: Formal LKI snapshot system (CR 608.2g) — current fallback covers ~95% but breaks on Oblivion Ring chains, flicker. Write edge case *tests* during PB work before fix lands.
- **M11**: Turn control (Mindslaver) + step skipping (Necropotence) — feature gaps, not architectural
- **Post-alpha (M16+)**: DSL escape hatch (`Effect::Custom(CardId)`) for irreducibly complex cards

**Why:** Reviewer correctly identified cost modifier bug as invariant violation — "everything flows through layers except this one thing." The longer it lives, the more cards get authored against the broken assumption. Bumped to early M10.

**How to apply:** When starting M10, do the engine correctness pass (cost modifiers, LKI, resolution suspension) *before* networking work. When writing PB tests, add LKI edge cases (Oblivion Ring chains, flicker) to document failures before the fix.
