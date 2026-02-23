# Corner Case Correctness Audit

> **Living document.** Re-audit after each milestone that adds mechanics or cards.
> Use `cr-coverage-auditor` agent for bulk re-checks.
>
> **Last audited**: 2026-02-23 (post-M9, pre-M9.4)

---

## Summary

| Status | Count | % |
|--------|-------|---|
| Covered | 15 | 43% |
| Partial | 9 | 26% |
| Gap | 11 | 31% |

**Existing card definition simplifications**: 12 (see § Card Definition Gaps below)

---

## Corner Case Coverage Table

| # | Name | CR Refs | Status | Evidence | Addressed In |
|---|------|---------|--------|----------|-------------|
| 1 | Humility + Opalescence | 613.8 | **COVERED** | `layers.rs:878` | M4 |
| 2 | Blood Moon + Urborg (BM newer) | 613.8 | **COVERED** | `layers.rs:1010` | M4 |
| 3 | Blood Moon + Urborg (BM older, dependency wins) | 613.8 | **COVERED** | `layers.rs:1072` | M4 |
| 4 | Yixlid Jailer + Anger | 613.1f, 112.6 | **GAP** | none | M9.4 |
| 5 | Clone copying a Clone | 707.2, 707.3 | **GAP** | none — layer 1 not implemented | M9.4 |
| 6 | Humility + Magus of the Moon | 613.8 | **PARTIAL** | `layers.rs:1382` — general layer ordering tested, specific non-dependency not named | M9.4 |
| 7 | Opalescence + Parallax Wave | 400.7, 613 | **PARTIAL** | `layers.rs:808` — type-change tested, zone-change interaction not | M9.4 |
| 8 | Deathtouch + Trample | 702.2, 702.19 | **COVERED** | `combat.rs:541` | M6 |
| 9 | Indestructible + Deathtouch | 702.12, 704.5h | **PARTIAL** | Each SBA tested separately; no combined test | M9.4 |
| 10 | Legendary Rule (Simultaneous Entry) | 704.5j, 603.6a | **PARTIAL** | SBA fires; ETB-before-removal not tested | M9.4 |
| 11 | +1/+1 and -1/-1 Counter Annihilation | 704.5q | **COVERED** | `sba.rs:558, 591` | M3 |
| 12 | Spell Fizzle (All Targets Illegal) | 608.2b | **COVERED** | `targeting.rs:297` | M5 |
| 13 | Partial Fizzle (Some Targets Illegal) | 608.2b | **COVERED** | `targeting.rs:386` | M5 |
| 14 | APNAP Trigger Ordering | 603.3b | **COVERED** | `abilities.rs:616`, `six_player.rs:222` | M7 |
| 15 | Panharmonicon Trigger Doubling | 603.2 | **GAP** | Trigger-modifier replacement not implemented | M9.4 |
| 16 | Multiple Replacement Effects (Player Choice) | 614.5 | **COVERED** | `replacement_effects.rs:745, 2568` | M8 |
| 17 | Self-Replacement Effects Apply First | 614.15 | **COVERED** | `replacement_effects.rs:525, 613` | M8 |
| 18 | Commander Zone-Change + Rest in Peace | 903.9a, 614 | **COVERED** | `replacement_effects.rs:1240, 1341` | M8–M9 |
| 19 | "Enters Tapped" Replacement | 614 | **COVERED** | `replacement_effects.rs:1952, 2134` | M8 |
| 20 | First Strike + Double Strike (Combined) | 702.4, 702.7 | **PARTIAL** | Each tested separately; combined interaction untested | M9.4 |
| 21 | Protection from X (DEBT) | 702.16 | **GAP** | Protection keyword not implemented | M9.4 |
| 22 | Hexproof vs. Non-Targeted Effects | 702.11 | **PARTIAL** | Blocks targeting tested; "does NOT block global" untested | M9.4 |
| 23 | "When This Dies" + Flicker | 400.7, 608.2b | **GAP** | Object identity exists; compound fizzle+no-trigger untested | M9.4 |
| 24 | Tokens Briefly in Non-Battlefield Zones | 111.8, 704.5d | **PARTIAL** | Token SBA tested; die-trigger-before-SBA window untested | M9.4 |
| 25 | Phasing and Auras/Equipment | 702.26 | **GAP** | Phasing not implemented | Deferred |
| 26 | Commander Damage from a Copy | 903 | **COVERED** | `commander_damage.rs:221` | M9 |
| 27 | Commander Tax with Partners | 903 | **COVERED** | `commander.rs:905` | M9 |
| 28 | Commander Dies with Exile Replacement | 903.9a, 614 | **COVERED** | `replacement_effects.rs:1240, 1341` | M8–M9 |
| 29 | Cascade into a Split Card | 702.84, 708.4 | **GAP** | Cascade not implemented; split-card MV not implemented | M9.4 |
| 30 | Morph/Manifest Face-Down | 708 | **GAP** | Face-down mechanics not implemented | Deferred |
| 31 | Aura on Illegal Permanent After Type Change | 704.5m | **PARTIAL** | Unattached aura SBA tested; type-change-induced falloff untested | M9.4 |
| 32 | Mutate Stack Ordering | 725 | **GAP** | Mutate not implemented | Deferred |
| 33 | Sylvan Library + Draw Replacement | 614 | **PARTIAL** | SkipDraw tested; "cards drawn this turn" tracking untested | M9.4 |
| 34 | Reveillark + Karmic Guide Loop | 726, 104.4b | **GAP** | Infinite loop detection not implemented | M9.4 |
| 35 | Storm + Copying | 702.40, 707.10 | **GAP** | Storm and spell-copy not implemented | M9.4 |

---

## Card Definition Gaps

Simplifications and no-op placeholders in the existing 54 card definitions.
These are correctness issues — the cards exist but behave incorrectly.

| Card(s) | Gap | Severity | CR Ref | Addressed In |
|---------|-----|----------|--------|-------------|
| Lightning Greaves, Swiftfoot Boots | Equipment doesn't grant keywords (shroud/hexproof/haste) via continuous effects | HIGH | 702.11, 702.10 | M9.4 |
| Alela, Cunning Conqueror | Goad is `DrawCards(0)` no-op placeholder | HIGH | 701.38 | M9.4 |
| Read the Bones | Scry simplified to nothing | MEDIUM | 701.18 | M9.4 |
| Dimir Guildgate | Modal color choice simplified to colorless | MEDIUM | 106.6 | M9.4 |
| Rogue's Passage | "Can't be blocked" effect not implemented | MEDIUM | 509.1a | M9.4 |
| Thought Vessel, Reliquary Tower | "No maximum hand size" not implemented | MEDIUM | 402.2 | M9.4 |
| Path to Exile | Optional search is unconditional | MEDIUM | 701.19 | M9.4 |
| Rhystic Study | Payment choice deferred — draw always fires | MEDIUM | — | M9.4 |
| Rest in Peace | ETB "exile all graveyards" not implemented | MEDIUM | — | M9.4 |
| Leyline of the Void | Opening hand rule not modeled | LOW | 113.6b | M9.4 |
| Darksteel Colossus | Shuffle-into-library is just redirect | LOW | 701.20 | M9.4 |
| Alela (trigger scoping) | Fires on all turns, not just opponent turns; no creature-type filter | LOW | 603.2 | M9.4 |

---

## Deferred Items

These require entire unimplemented subsystems. They will be addressed when those
subsystems are built, likely as part of M12+ card pipeline expansion.

| # | Name | Mechanic | Rationale |
|---|------|----------|-----------|
| 25 | Phasing + Auras/Equipment | Phasing (CR 702.26) | Rare in Commander; entire subsystem needed |
| 30 | Morph/Manifest Face-Down | Face-down (CR 708) | Entire subsystem needed |
| 32 | Mutate Stack Ordering | Mutate (CR 725) | Ikoria-specific; entire subsystem needed |

---

## Audit Process

1. Read `docs/mtg-engine-corner-cases.md` (35 cases)
2. Grep `crates/engine/tests/` for test functions covering each case
3. Grep `test-data/generated-scripts/` for scripts covering each case
4. Mark: COVERED / PARTIAL / GAP
5. For existing card definitions: scan `crates/engine/src/cards/definitions.rs` for TODO comments, simplifications, and no-op placeholders
6. Update this document with findings

**Trigger**: Run after every milestone that adds mechanics, keywords, or card definitions.
