# MTG Engine — Known Interaction Gaps

> Tracked gaps discovered by testing the engine against known confusing MTG rules interactions.
> These are not milestone-scoped — fix opportunistically or when a card needs the behavior.
>
> **Created**: 2026-03-13
> **Last Updated**: 2026-03-13

---

## Status Definitions

| Status | Meaning |
|--------|---------|
| `open` | Known gap, not yet addressed |
| `in-progress` | Actively being worked on |
| `fixed` | Resolved with tests |
| `wont-fix` | Not relevant to Commander or too niche to justify |

---

## IG-1: ETB Trigger Collection Ignores Layer 6 Ability Removal

| Field | Value |
|-------|-------|
| **Status** | `fixed` |
| **Severity** | HIGH |
| **CR** | 603.2, 613 (Layer 6) |
| **Canonical Example** | Dress Down + Evoked Solitude |
| **File** | `crates/engine/src/rules/abilities.rs:5784` |
| **Root Cause** | `queue_carddef_etb_triggers()` reads ETB triggers from `obj.characteristics.triggered_abilities` — the card's printed abilities cached at zone entry. It does not call `calculate_characteristics()` to check whether continuous effects (e.g. Dress Down's Layer 6 "remove all abilities") have suppressed the ability. |
| **Correct Behavior** | When queueing ETB triggers, the engine should resolve the permanent's current characteristics through the layer system. If a Layer 6 effect has removed the ability, the trigger should not be queued. |
| **Impact** | Any card that removes abilities as a static effect (Dress Down, Humility, Overwhelming Splendor, Cursed Totem for activated abilities) will fail to suppress ETB triggers on newly-entering creatures. |
| **Fix Approach** | In `queue_carddef_etb_triggers()`, call `calculate_characteristics()` for the entering permanent and read triggered abilities from the result, not from the cached `obj.characteristics`. Performance: one extra layer resolution per ETB — acceptable since ETBs are infrequent relative to game actions. |
| **Related** | Morph face-down ETB suppression (IG-1a) is handled as a special case in the same function. A general fix here would subsume that special case. |
| **Cards Affected** | Dress Down, Humility (when entering same time as creatures), Overwhelming Splendor, Strict Proctor (partial overlap with IG-2) |

---

## IG-2: No DSL Support for Static ETB Trigger Suppression (Torpor Orb Pattern)

| Field | Value |
|-------|-------|
| **Status** | `fixed` |
| **Severity** | MEDIUM |
| **CR** | 614.16a (Torpor Orb creates a replacement effect that prevents ETB triggered abilities from triggering) |
| **Canonical Example** | Torpor Orb + Phyrexian Dreadnought |
| **File** | `crates/engine/src/cards/card_definition.rs` (Effect enum), `crates/engine/src/rules/abilities.rs` (trigger collection) |
| **Root Cause** | The Effect DSL has no variant for "creatures entering the battlefield don't cause abilities to trigger." There's no mechanism for a static ability on one permanent to suppress triggers on other permanents. |
| **Correct Behavior** | When a permanent with a Torpor Orb-style static ability is on the battlefield, creatures entering the battlefield should not have their ETB triggered abilities placed on the stack. This is a replacement effect (CR 614.16a), not a counter — the trigger never fires, rather than being countered after firing. |
| **Impact** | Cannot author card definitions for: Torpor Orb, Hushbringer, Hushwing Gryff, Tocatli Honor Guard, Strict Proctor (partial — Strict Proctor taxes rather than prevents). |
| **Fix Approach** | Two-part fix: (1) Add an `Effect::SuppressETBTriggers` or a `StaticAbility::PreventETBTriggers` variant to the DSL. (2) In `queue_carddef_etb_triggers()` (and `flush_pending_triggers()`), scan the battlefield for permanents with this static ability and skip trigger queueing for affected permanents. Respect the distinction between "creatures" and "all permanents" (Torpor Orb = creatures only; a hypothetical future card might affect all permanents). |
| **Solvable** | Yes. The trigger collection path (`abilities.rs:5784`) is a single chokepoint. Adding a battlefield scan for suppression effects before queueing is straightforward. The DSL needs one new variant; the enforcement needs ~20 lines in the trigger collection path. |
| **Cards Affected** | Torpor Orb, Hushbringer, Hushwing Gryff, Tocatli Honor Guard, Strict Proctor |

---

## IG-3: Torpor Orb vs. Stifle Distinction Not Testable

| Field | Value |
|-------|-------|
| **Status** | `open` |
| **Severity** | LOW |
| **CR** | 614.16a vs. 603.1 |
| **Canonical Example** | Torpor Orb vs. Stifle on Phyrexian Dreadnought |
| **Root Cause** | The engine can counter triggered abilities on the stack (Stifle pattern works), but the Torpor Orb pattern (IG-2) is not implementable. Until IG-2 is fixed, the engine cannot distinguish between "trigger never fires" and "trigger fires but is countered." |
| **Impact** | Low — the distinction matters for cards that care about triggers being put on the stack (e.g. Strict Proctor taxes triggers; if the trigger never fires, there's nothing to tax). |
| **Depends On** | IG-2 |

---

## Verified Correct

These interactions were checked and the engine handles them correctly:

| Interaction | CR | Engine Location | Test |
|-------------|-----|-----------------|------|
| Humility + Opalescence (timestamp-dependent layers) | 613.7 | `rules/layers.rs:48-60` | `tests/layers.rs:871` |
| Blood Moon + Urborg (CR 613.8 dependency) | 613.8 | `rules/layers.rs:832-863` | `tests/layers.rs` (dependency tests) |
| P/T Switching sublayer ordering (7a-7d) | 613.4 | `state/continuous_effect.rs:23-44` | `tests/layers.rs` |
| Countering triggered abilities on stack (Stifle) | 603.1 | `rules/resolution.rs` | `tests/resolution.rs:2350+` |
| Phasing out Platinum Angel (protection ceases) | 702.26 | `rules/abilities.rs` (phasing) | `tests/phasing.rs` |
