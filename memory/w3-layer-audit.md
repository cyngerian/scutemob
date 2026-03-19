---
name: W3-LC Layer Correctness Audit
description: Audit and fix all base-characteristic reads that should use calculate_characteristics() — silent rules violations under Humility/Dress Down/Painter's Servant
type: project
---

# W3-LC: Layer Correctness Audit

**Created**: 2026-03-19
**Workstream**: W3 (LOW Remediation), subpath LC (Layer Correctness)
**Commit prefix**: `W3-LC:`
**Independent of**: W6 (Primitive + Card Authoring) — no shared code paths

**Why:** Early ability batches (B0-B3, ~2026-03-01) and some M0-M9 engine code read
`obj.characteristics.keywords` (base/printed) instead of `calculate_characteristics(state, id)`
(layer-resolved). Any continuous effect that adds or removes abilities (Humility, Dress Down,
equipment grants, Aura grants) produces wrong game state at these sites. Discovered during
a rules audit of Flanking (B2) — the trigger fires even under Humility.

**How to apply:** Each site must be classified, then fixed if needed. Fix = replace base read
with `calculate_characteristics()` call + add a Humility interaction test.

---

## Scope: 69 base-characteristic reads across 12 files

| File | Count | Priority | Notes |
|------|-------|----------|-------|
| `effects/mod.rs` | 25 | MEDIUM | Effect execution — filters, targeting |
| `resolution.rs` | 15 | HIGH | Trigger/spell resolution — wrong chars = wrong game state |
| `abilities.rs` | 7 | HIGH | Trigger checks — Flanking bug confirmed here |
| `replacement.rs` | 5 | MEDIUM | Replacement effects fire during events |
| `sba.rs` | 3 | MEDIUM | "Is this a creature with 0 toughness?" needs layer types |
| `casting.rs` | 3 | LOW | Stack objects, not battlefield — usually correct |
| `engine.rs` | 3 | LOW | Command dispatch |
| `copy.rs` | 2 | LOW | Copiable values — base reads may be intentional (CR 706.2) |
| `mana.rs` | 2 | LOW | Mana abilities — narrow scope |
| `state/mod.rs` | 2 | LOW | Zone transitions |
| `replay_harness.rs` | 1 | SKIP | Test infrastructure |
| `layers.rs` | 1 | SKIP | Correct by definition |

**Also counts**: 139 existing `calculate_characteristics()` calls — the engine already
does this correctly in most places. The 69 sites are the gap.

---

## Classification Guide

Each site gets one of:

- **`correct-base`** — Intentionally reads base characteristics:
  - Inside `layers.rs` (building the layer result)
  - Copy effects reading copiable values (CR 706.2)
  - Objects not yet on the battlefield (stack, hand, exile without CDAs)
  - Card registry / definition lookups (not game state)

- **`needs-layer-calc`** — Bug. Must use `calculate_characteristics()`:
  - Battlefield objects checked for keywords/types during triggers
  - Battlefield objects checked during SBA evaluation
  - Battlefield objects checked during effect execution
  - Any object checked for CDAs (Devoid, Changeling) in non-battlefield zones

- **`ambiguous`** — Needs case-by-case CR analysis

---

## Session Plan

### Session 1: Audit + Classify (read-only)
- Read every site in priority order (HIGH → MEDIUM → LOW)
- Classify each as `correct-base`, `needs-layer-calc`, or `ambiguous`
- Fill in the Per-File Audit section below
- No code changes

### Session 2: Fix HIGH (`abilities.rs` + `resolution.rs`)
- Fix all `needs-layer-calc` sites
- Add Humility interaction test for each fixed trigger/resolution path
- `cargo test --all` + `cargo clippy -- -D warnings`

### Session 3: Fix MEDIUM (`effects/mod.rs`, `replacement.rs`, `sba.rs`)
- Same pattern
- `effects/mod.rs` has 25 sites — may split if many need fixing

### Session 4: Fix LOW + regression test
- Remaining files
- Add property test or grep-based CI check to prevent regression

---

## Per-File Audit

### abilities.rs (7 sites) — Priority: HIGH

| Line | Expression | Classification | Notes |
|------|-----------|---------------|-------|
| 706 | `obj.characteristics.keywords` | | |
| 3444 | `obj.characteristics.card_types` | | |
| 3446 | `obj.characteristics.keywords` | | |
| 4311 | Flanking attacker check | **needs-layer-calc** | Confirmed bug: Humility breaks it |
| 4323 | Flanking blocker check | **needs-layer-calc** | Confirmed bug: granted Flanking ignored |
| 4435 | `obj.characteristics.keywords` | | |
| 6376 | `obj.characteristics.keywords` | | |
| 6542-6543 | `obj.characteristics.card_types` (×2) | | |

### resolution.rs (15 sites) — Priority: HIGH

| Line | Expression | Classification | Notes |
|------|-----------|---------------|-------|
| 632 | `obj.characteristics.card_types` (suspend haste) | | On ETB — just entered |
| 1795 | `obj.characteristics.card_types` | | |
| 3661-3662 | `obj.characteristics.card_types` (×2) | | |
| 4005 | `obj.characteristics.card_types` | | |
| 5159 | `o.characteristics.card_types` | | |
| 5633 | `obj.characteristics.card_types` | | |
| *(remaining 8 — locate during Session 1)* | | | |

### effects/mod.rs (25 sites) — Priority: MEDIUM

*(Enumerate during Session 1)*

### replacement.rs (5 sites) — Priority: MEDIUM

| Line | Expression | Classification | Notes |
|------|-----------|---------------|-------|
| 419 | `o.characteristics.card_types` | | |
| 424 | `o.characteristics.card_types` | | |
| 1642 | `o.characteristics.card_types` | | |
| *(remaining 2 — locate during Session 1)* | | | |

### sba.rs (3 sites) — Priority: MEDIUM

*(Enumerate during Session 1)*

### casting.rs, engine.rs, copy.rs, mana.rs, state/mod.rs — Priority: LOW

*(Enumerate during Session 1)*

---

## Performance Note

`calculate_characteristics()` clones the base `Characteristics` and iterates all active
continuous effects. It's not free. For hot paths (SBA checks run every priority pass),
consider caching or batching. However, correctness > performance — fix first, optimize
if benchmarks regress.
