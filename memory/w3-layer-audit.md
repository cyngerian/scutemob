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

## S1 Audit Summary (2026-03-19)

**Total standalone reads audited**: ~110
**needs-layer-calc (bugs)**: 46 (43 original + Soulbond 3444-3446 + CardCount battlefield path + abilities.rs 6053-6056)
**correct-base**: ~64 (+ replacement.rs 1187 resolved from ambiguous)
**ambiguous**: 0 (all resolved)

### Bug Distribution by Severity

**HIGH** (visibly wrong results in normal gameplay):
- `effects/mod.rs:3730,3747` — PowerOf/ToughnessOf reads base P/T, ignoring counters/equipment/anthems
- `abilities.rs:6035` + `6053-6056` — collect_triggers_for_event reads base triggered abilities — Humility doesn't suppress triggers; ETB filter also reads base card_types
- `mana.rs:154,157-159,181` — summoning sickness ignores animated permanents and layer-granted haste

**MEDIUM** (require uncommon but real interactions):
- `abilities.rs:222,246` — activated ability access bypasses Humility
- `abilities.rs:706,6376` — hexproof/shroud/protection bypasses layer removal
- `abilities.rs:4311,4323` — Flanking checks bypass layers (confirmed Humility bug)
- `abilities.rs:6542-6543` — artifact creature check for Modular
- `resolution.rs:1795,3661-3662,5159` — battlefield type checks (Cipher, Reconfigure, Ninjutsu)
- `effects/mod.rs:268,673,855,2115` — destroy/damage/sacrifice type capture
- `effects/mod.rs:3536` — AllCreatures filter
- `effects/mod.rs:2304-2306` — ChooseCreatureType scan
- `effects/mod.rs` condition block (4489-4643) — ETB-tapped land checks under Blood Moon
- `replacement.rs:419,424` — object filter for replacement effects
- `replacement.rs:1590-1595` — ChooseCreatureType scan
- `sba.rs:932-939` — Legend rule supertype+name
- `sba.rs:1047` — Aura pre-filter
- `casting.rs:5182` — hexproof/protection targeting
- `engine.rs:2262` — ring-bearer selection

**LOW** (theoretically possible, extremely unlikely):
- `effects/mod.rs` matches_filter callers for specific conditions

### Key Structural Finding

`matches_filter()` in effects/mod.rs takes `&Characteristics` — the function itself is correct.
The bug is at **call sites** that pass `&obj.characteristics` for battlefield permanents instead
of layer-calculated characteristics. A single fix strategy: calculate characteristics at each
call site and pass the resolved version.

---

## Session Plan (revised after S1)

### Session 1: Audit + Classify (read-only) — COMPLETE
- Classified all sites across 10 files
- Found 43 bugs, 3 ambiguous, ~64 correct

### Session 2: Fix HIGH sites (3 files, 7 sites) — COMPLETE
- `effects/mod.rs:3730,3747` — PowerOf/ToughnessOf: zone-aware layer calc (battlefield → layer-resolved, other → base)
- `abilities.rs:6035` — collect_triggers_for_event: use layer-resolved triggered_abilities (Humility suppresses)
- `abilities.rs:6053-6056` — ETB filter creature_only: use layer-resolved card_types (animated lands recognized)
- `mana.rs:154,157-159` — summoning sickness: layer-resolved creature type + haste (Fervor, animated lands)
- `mana.rs:181` — sacrifice creature check: layer-resolved types (animated artifacts emit CreatureDied)
- 8 new tests in `crates/engine/tests/layer_correctness.rs` (Humility, animation, Fervor, anthem interactions)
- 2183 tests passing, 0 clippy warnings

### Session 3: Fix MEDIUM sites — abilities.rs + resolution.rs + casting.rs + engine.rs (17 sites) — COMPLETE
- `abilities.rs:222,246` — activated ability access uses layer-resolved (Humility removes activated abilities)
- `abilities.rs:706→722` — hexproof/shroud/protection target check uses layer-resolved keywords
- `abilities.rs:3444-3466` — Soulbond pairing uses layer-resolved types + keywords (replaced partial OR fallback)
- `abilities.rs:4311,4323` — Flanking attacker/blocker checks use layer-resolved keywords
- `abilities.rs:6376→6403` — hexproof/shroud/protection target check reordered to use pre-computed layer chars
- `abilities.rs:6542→6571-6572` — Modular artifact creature target uses layer-resolved types
- `resolution.rs:1795` — Cipher creature attachment uses layer-resolved types
- `resolution.rs:1939,2077,2114` — triggered ability resolution uses layer-resolved triggered_abilities (namespace alignment with collect_triggers_for_event)
- `resolution.rs:3683-3684` — Modular target resolution uses layer-resolved types
- `resolution.rs:5181` — Ninjutsu creature target uses layer-resolved types
- `casting.rs:5182` — hexproof/protection targeting uses layer-resolved keywords
- `engine.rs:2262` — ring-bearer creature selection uses layer-resolved types
- 2183 tests passing, 0 clippy warnings

### Session 4: Fix MEDIUM sites — effects/mod.rs (22 sites) — COMPLETE
- DealDamage type capture (268) — layer-resolved Planeswalker/Creature check
- DestroyTarget type capture (673, 855) — layer-resolved pre-zone-move types
- SacrificeAll type capture (2115) — layer-resolved pre-zone-move types
- ChooseCreatureType (2321-2323) — layer-resolved types + subtypes
- AllCreatures filter (3558) — layer-resolved creature check
- matches_filter callers — DestroyAll, ExileAll, AllPermanentsMatching, PermanentCount, YouControlPermanent, OpponentControlsPermanent, EachPermanentMatching (7 sites)
- CardCount zone-conditional (3851) — battlefield uses layer-resolved, other zones use base
- ForEach creature targets — EachCreature, EachCreatureYouControl, EachOpponentsCreature (3 sites)
- Condition block — ControlLandWithSubtypes, ControlAtMostNOtherLands, ControlBasicLandsAtLeast, ControlAtLeastNOtherLands, ControlAtLeastNOtherLandsWithSubtype, ControlLegendaryCreature, ControlCreatureWithSubtype (7 sites)
- 2183 tests passing, 0 clippy warnings

### Session 5: Fix MEDIUM sites — replacement.rs + sba.rs (7 sites) — COMPLETE
- `replacement.rs:419,424` — object_matches_filter AnyCreature/HasCardType uses layer-resolved types
- `replacement.rs:1590-1606` — ChooseCreatureType scan uses layer-resolved types + subtypes
- `sba.rs:932-939` — Legend rule uses layer-resolved supertypes + name (copy effects change name)
- `sba.rs:1047` — Aura pre-filter reuses already-computed layer-resolved chars
- 2183 tests passing, 0 clippy warnings

### Session 6: Regression prevention — DEFERRED (no CI infrastructure)
- No CI pipeline exists yet. A grep-based lint for `obj.characteristics.` on battlefield
  objects would be the right regression gate, but it should be part of a broader CI setup
  (likely M10 timeframe when networking makes automated checks more valuable).
- **Pattern to catch**: `obj.characteristics.(card_types|keywords|subtypes|supertypes|power|toughness)`
  reads on objects known to be on the battlefield — should use `calculate_characteristics()` instead.
- Revisit when CI is set up. Until then, this audit doc serves as the reference for reviewers.

---

## Per-File Audit

### abilities.rs — Priority: HIGH

**Standalone reads (not fallback patterns):**

| Line | Expression | Classification | Notes |
|------|-----------|---------------|-------|
| 222 | `obj.characteristics.activated_abilities[ability_index]` | **needs-layer-calc** | Activated ability sorcery_speed check on battlefield permanent; Humility removes abilities |
| 246 | `obj.characteristics.activated_abilities[ability_index]` | **needs-layer-calc** | Cloning cost/effect of activated ability; downstream of 222 |
| 706 | `obj.characteristics.keywords` | **needs-layer-calc** | Hexproof/shroud/protection check on target; Humility removes these |
| 2476 | `obj.characteristics.name.clone()` | correct-base | Object in graveyard; names don't change in normal play |
| 3444-3446 | `obj.characteristics.card_types` + `.keywords` | **needs-layer-calc** | Soulbond pairing — card_types bypasses layers (bug); keyword has partial OR-fallback but short-circuits on base (false positive path). Review resolved: MEDIUM |
| 3964 | `obj.characteristics.triggered_abilities.get(t.ability_index)` | correct-base | Post-processing already-queued triggers (Myriad); CR 113.7a |
| 3987 | `obj.characteristics.triggered_abilities.get(t.ability_index)` | correct-base | Already-queued triggers (Provoke) |
| 4031 | `obj.characteristics.triggered_abilities.get(t.ability_index)` | correct-base | Already-queued triggers (Melee) |
| 4065 | `obj.characteristics.triggered_abilities.get(t.ability_index)` | correct-base | Already-queued triggers (Enlist) |
| 4311 | Flanking attacker check | **needs-layer-calc** | Confirmed bug: Humility breaks it (pre-existing) |
| 4323 | Flanking blocker check | **needs-layer-calc** | Confirmed bug: granted Flanking ignored (pre-existing) |
| 4429 | `obj.characteristics.triggered_abilities.get(t.ability_index)` | correct-base | Already-queued triggers (Rampage) |
| 4435 | `obj.characteristics.keywords` | correct-base | Extracting Rampage(n) value from already-queued trigger |
| 4520 | `obj.characteristics.triggered_abilities.iter().enumerate()` | correct-base | SelfDies trigger — object in graveyard (LKI) |
| 4901 | `obj.characteristics.triggered_abilities.iter().enumerate()` | correct-base | Aura dies trigger — object in graveyard |
| 5037 | `obj.characteristics.triggered_abilities.iter().enumerate()` | correct-base | Connive trigger — non-battlefield zone |
| 6035 | `obj.characteristics.triggered_abilities.iter().enumerate()` | **needs-layer-calc** | **collect_triggers_for_event()** — general trigger collector for battlefield permanents; Humility should suppress |
| 6376 | `obj.characteristics.keywords` | **needs-layer-calc** | Hexproof/shroud/protection on battlefield permanent |
| 6542-6543 | `obj.characteristics.card_types` (×2) | **needs-layer-calc** | Modular target — artifact creature check on battlefield |

**Fallback reads (correct — skipped):** 259, 298, 336, 370, 556, 627, 694, 3328, 3538, 3555, 3591, 3707, 3733, 6293, 6386, 7275, 7374, 7454, 7659, 7738

**Totals**: 8 needs-layer-calc (incl. 2 pre-existing Flanking + 3444-3446 resolved from ambiguous) + 10 correct-base + 1 missed site (6053-6056, fix with 6035)

### resolution.rs — Priority: HIGH

| Line | Expression | Classification | Notes |
|------|-----------|---------------|-------|
| 140 | `card.characteristics.card_types.clone()` | correct-base | Stack object (spell resolving) |
| 600, 604 | `obj.characteristics.keywords.insert(Haste)` | correct-base | Mutation (Dash/Blitz haste grant) |
| 610 | `obj.characteristics.triggered_abilities.push(...)` | correct-base | Mutation (Blitz draw trigger) |
| 632 | `obj.characteristics.card_types.contains(Creature)` | correct-base | Just-created ETB object, pre-layer-calc |
| 650-654 | `obj.characteristics.power/toughness/colors/mana_cost` | correct-base | Mutation (Prototype) |
| 1608-1609 | `obj.characteristics.card_types.remove/insert` | correct-base | Mutation (Bestow type change) |
| 1795 | `obj.characteristics.card_types.contains(Creature)` | **needs-layer-calc** | Battlefield permanent type check (Cipher attachment) |
| 1855 | `obj.characteristics.activated_abilities.get(idx)` | correct-base | Ability template lookup (stack, CR 113.7a) |
| 1939, 2068, 2098 | `obj.characteristics.triggered_abilities.get(idx)` | correct-base | Trigger template lookup (stack) |
| 3007 | `obj.characteristics.keywords.insert(Haste)` | correct-base | Mutation (Unearth haste grant) |
| 3661-3662 | `obj.characteristics.card_types.contains(...)` (×2) | **needs-layer-calc** | Battlefield type check (Reconfigure target) |
| 5159 | `o.characteristics.card_types.contains(Creature)` | **needs-layer-calc** | Battlefield type check (Ninjutsu target) |
| 5633 | `obj.characteristics.card_types.contains(Creature)` | correct-base | Stack object (Suspend free-cast) |
| 5854 | `obj.characteristics.name` | correct-base | Library search by name |
| 7019 | `target_obj.characteristics = top.characteristics.clone()` | correct-base | Mutation (Mutate merge) |

**Fallback reads (correct — skipped):** 1003, 3725, 3735, 4005/4008, 4405, 4565, 5372, 6890, 6936, 6964, 7192

**Totals**: 3 needs-layer-calc + 12 correct-base

### effects/mod.rs — Priority: MEDIUM

| Line | Expression | Classification | Notes |
|------|-----------|---------------|-------|
| 268 | `.characteristics.card_types.clone()` | **needs-layer-calc** | DealDamage — battlefield permanent Planeswalker/Creature check |
| 673 | `.characteristics.card_types.clone()` | **needs-layer-calc** | DestroyTarget — pre-zone-move type capture |
| 855 | `.characteristics.card_types.clone()` | **needs-layer-calc** | DestroyTarget variant — pre-zone-move type capture |
| 1670-1671 | `obj.characteristics.subtypes` (mutation) | correct-base | Amass — intentional base modification |
| 2115 | `obj.characteristics.card_types.clone()` | **needs-layer-calc** | SacrificeAll — pre-zone-move type capture |
| 2304-2306 | `.characteristics.card_types/subtypes` | **needs-layer-calc** | ChooseCreatureType — battlefield creature scan |
| 2852 | `.characteristics.card_types.contains(Land)` | correct-base | Hideaway play — card in exile |
| 2888 | `&obj.characteristics.card_types` | correct-base | Hideaway cast — card in exile |
| 3284 | `.characteristics.card_types.contains(Land)` | correct-base | Connive discard — card in hand |
| 3536 | `.characteristics.card_types.contains(Creature)` | **needs-layer-calc** | AllCreatures filter — battlefield |
| 3712 | `obj.characteristics.power` | **needs-layer-calc** | **PowerOf** — ignores counters/equipment/anthems (HIGH) |
| 3729 | `obj.characteristics.toughness` | **needs-layer-calc** | **ToughnessOf** — ignores counters/equipment/anthems (HIGH) |
| 4489, 4492 | `.characteristics.card_types/subtypes` | **needs-layer-calc** | ControlLandWithSubtypes — battlefield |
| 4505 | `.characteristics.card_types.contains(Land)` | **needs-layer-calc** | ControlAtMostNOtherLands — battlefield |
| 4528 | `.characteristics.subtypes.contains(st)` | correct-base | CanRevealFromHandWithSubtype — hand |
| 4541 | `.characteristics.card_types.contains(Land)` | **needs-layer-calc** | ControlBasicLandsAtLeast — battlefield |
| 4561 | `.characteristics.card_types.contains(Land)` | **needs-layer-calc** | ControlAtLeastNOtherLands — battlefield |
| 4577 | `.characteristics.subtypes.contains(subtype)` | **needs-layer-calc** | ControlAtLeastNOtherLandsWithSubtype — battlefield |
| 4587 | `.characteristics.card_types.contains(Creature)` | **needs-layer-calc** | ControlLegendaryCreature — battlefield |
| 4598-4599 | `.characteristics.card_types/subtypes` | **needs-layer-calc** | ControlCreatureWithSubtype — battlefield |
| 4621 | `.characteristics.card_types.contains(Creature)` | **needs-layer-calc** | EachCreature — battlefield |
| 4632 | `.characteristics.card_types.contains(Creature)` | **needs-layer-calc** | EachCreatureYouControl — battlefield |
| 4643 | `.characteristics.card_types.contains(Creature)` | **needs-layer-calc** | EachOpponentsCreature — battlefield |

**Fallback reads (correct — skipped):** 2981, 3096, 3129

**Additional matches_filter callers passing base characteristics** (battlefield):
- Lines ~799 (DestroyAll), ~975 (ExileAll), ~3552 (AllPermanentsMatching), ~3796 (PermanentCount), ~4394 (YouControlPermanent), ~4400 (OpponentControlsPermanent), ~4656 (EachPermanentMatching)

**Zone-conditional:** Line ~3780 (CardCount — correct-base for non-battlefield zones, needs-layer-calc when zone=Battlefield; fix: check zone_id and conditionally calculate)

**Totals**: 22+ needs-layer-calc + 5 correct-base + 1 ambiguous

### replacement.rs — Priority: MEDIUM

| Line | Expression | Classification | Notes |
|------|-----------|---------------|-------|
| 419 | `o.characteristics.card_types.contains(Creature)` | **needs-layer-calc** | object_matches_filter — battlefield type check for replacement applicability |
| 424 | `o.characteristics.card_types.contains(ct)` | **needs-layer-calc** | HasCardType filter — same function |
| 549 | `obj.characteristics.keywords.iter().find_map(...)` | correct-base | Dredge — object in graveyard |
| 1187 | `.map(|o| o.characteristics.clone())` | correct-base | Entering-battlefield object — base types represent what card "is" before entry. Review resolved: correct-base, LOW risk |
| 1590-1595 | `.characteristics.card_types/subtypes` | **needs-layer-calc** | ChooseCreatureType scan — battlefield |
| 1642 | `o.characteristics.card_types.contains(Creature)` | correct-base | Object just moved to graveyard |

**Totals**: 4 needs-layer-calc + 3 correct-base (1187 resolved from ambiguous)

### sba.rs — Priority: MEDIUM

| Line | Expression | Classification | Notes |
|------|-----------|---------------|-------|
| 932-935 | `obj.characteristics.supertypes.contains(Legendary)` | **needs-layer-calc** | Legend rule — type-changing effects could grant/remove Legendary |
| 939 | `obj.characteristics.name.clone()` | **needs-layer-calc** | Legend rule groups by name — copy effects (Layer 1) can change name |
| 1047 | `obj.characteristics.subtypes.contains(Aura)` | **needs-layer-calc** | Aura SBA pre-filter — type-changing effects |
| 1088 | `.map(|o| o.characteristics.clone())` | correct-base | Fallback for calculate_characteristics at 1084 |
| 1144 | `obj.characteristics.subtypes.remove(Aura)` | correct-base | Mutation (Bestow revert) |
| 1149 | `obj.characteristics.card_types.insert(Creature)` | correct-base | Mutation (Bestow revert) |

**Totals**: 3 needs-layer-calc + 3 correct-base

### casting.rs — Priority: LOW

| Line | Expression | Classification | Notes |
|------|-----------|---------------|-------|
| 502 | `card_obj.characteristics.mana_cost.clone()` | correct-base | Card being cast from hand/graveyard |
| 3826-3831 | `stack_source.characteristics.*` (mutations) | correct-base | Prototype base char modifications |
| 5182 | `&obj.characteristics.keywords` | **needs-layer-calc** | Hexproof/protection targeting check — layer effects missed |
| 6320 | `obj.characteristics.card_types` | correct-base | Graveyard card type (Delirium) |

**All other casting.rs lines**: confirmed fallback patterns for nearby `calculate_characteristics` calls.

**Totals**: 1 needs-layer-calc + many correct-base

### engine.rs — Priority: LOW

| Line | Expression | Classification | Notes |
|------|-----------|---------------|-------|
| 1329 | `mat_obj.characteristics.card_types.contains(type)` | correct-base | Fallback for calculate_characteristics at 1327 (Craft battlefield material) |
| 1331 | `mat_obj.characteristics.card_types.contains(type)` | correct-base | Non-battlefield Craft material (graveyard/exile) |
| 2262 | `obj.characteristics.card_types.contains(Creature)` | **needs-layer-calc** | Ring temptation — battlefield creature selection |

**Totals**: 1 needs-layer-calc + 2 correct-base

### mana.rs — Priority: LOW → upgraded to HIGH

| Line | Expression | Classification | Notes |
|------|-----------|---------------|-------|
| 154 | `obj.characteristics.card_types.contains(Creature)` | **needs-layer-calc** | Summoning sickness — animated permanents bypass check |
| 157-159 | `obj.characteristics.keywords.contains(Haste)` | **needs-layer-calc** | Summoning sickness — layer-granted haste (Fervor) ignored |
| 181 | `obj.characteristics.card_types.contains(Creature)` | **needs-layer-calc** | Pre-sacrifice creature check — same animation issue |

**Totals**: 3 needs-layer-calc

### copy.rs — all correct-base

| Line | Expression | Classification | Notes |
|------|-----------|---------------|-------|
| 60 | `obj.characteristics.clone()` | correct-base | Copy-chain depth limit fallback |
| 66 | `obj.characteristics.clone()` | correct-base | Copy starting point (CR 706.2) |
| 374, 386, 583, 595 | card_types/mana_cost | correct-base | Fallback patterns for cascade/discover (exile zone) |

### protection.rs — correct-base

| Line | Expression | Classification | Notes |
|------|-----------|---------------|-------|
| 118 | `.map(|o| o.characteristics.clone())` | correct-base | Fallback for calculate_characteristics at 114 |

### state/mod.rs — SKIP (zone transitions, object creation)
### layers.rs — SKIP (layer building, correct by definition)
### replay_harness.rs — SKIP (test infrastructure)

---

## Performance Note

`calculate_characteristics()` clones the base `Characteristics` and iterates all active
continuous effects. It's not free. For hot paths (SBA checks run every priority pass),
consider caching or batching. However, correctness > performance — fix first, optimize
if benchmarks regress.

Key optimization opportunity: `matches_filter` callers in effects/mod.rs could batch
calculate characteristics for all battlefield objects once, then filter the pre-computed list.
