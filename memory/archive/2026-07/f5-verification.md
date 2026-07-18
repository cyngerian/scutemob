# F-5 Verification: Spot-Check of F-4 Session Fixes

**Reviewed**: 2026-03-22
**Cards**: 8 (substitutions noted below)
**Findings**: 1 HIGH, 1 MEDIUM, 2 LOW

## Substitutions

Five of the originally requested cards do not have definition files:
- `ranger_of_eos.rs` -- does not exist
- `briarbridge_patrol.rs` -- does not exist
- `spectacle_mage.rs` -- does not exist
- `jungle_basin.rs` -- does not exist
- `murkwater_pathway.rs` -- does not exist (card is filed as `clearwater_pathway.rs`, the front face)

Replacements drawn from the git-modified files in F-4 sessions:
- Bloomvine Regent (F-4 ETB trigger fix)
- Den of the Bugbear (F-4 conditional ETB land fix)
- Oathsworn Vampire (F-4 ETB tapped creature fix)
- Marang River Regent (F-4 bounce ETB TODO)
- Clearwater Pathway (stands in for Murkwater Pathway -- same file)

---

## Card 1: Shrieking Drake

**File**: `crates/engine/src/cards/defs/shrieking_drake.rs`
**Session**: F-4 session 2 (ETB bounce)

- **Oracle match**: YES
- **Types match**: YES (Creature -- Drake)
- **Mana cost match**: YES ({U})
- **P/T match**: YES (1/1)
- **DSL correctness**: YES
- **Findings**: None.
  - ETB trigger correctly targets a creature you control (`TargetController::You`).
  - `MoveZone` to `ZoneTarget::Hand` with `OwnerOf` targeting correctly returns to owner's hand (not controller's hand), which matches oracle "its owner's hand."
  - Flying keyword present.

**Verdict: PASS**

---

## Card 2: Brightclimb Pathway // Grimclimb Pathway

**File**: `crates/engine/src/cards/defs/brightclimb_pathway.rs`
**Session**: F-4 session 5 (pathway lands)

- **Oracle match**: PARTIAL -- see F1 below
- **Types match**: YES (Land)
- **Mana cost match**: YES (none)
- **DSL correctness**: YES (for front face)
- **Findings**:
  - F1 (LOW): Oracle text in def is `"{T}: Add {W}."` which is only the front face text. The full card is an MDFC with back face Grimclimb Pathway ("{T}: Add {B}."). Since the engine models MDFCs as front-face-only CardDefinitions (no `back_face` field populated), this is the expected pattern for pathway lands. The back face is not modeled, which means Grimclimb Pathway cannot be played. This is a known limitation of MDFC support, not a per-card bug.
  - Mana ability correct: `mana_pool(1,0,0,0,0,0)` = {W}. WUBRGC order verified.

**Verdict: PASS** (known MDFC limitation, not a card-specific issue)

---

## Card 3: Clearwater Pathway // Murkwater Pathway

**File**: `crates/engine/src/cards/defs/clearwater_pathway.rs`
**Session**: F-4 session 5 (pathway lands)

- **Oracle match**: PARTIAL -- same MDFC front-face-only pattern as Brightclimb
- **Types match**: YES (Land)
- **Mana cost match**: YES (none)
- **DSL correctness**: YES (for front face)
- **Findings**: Same MDFC limitation as Brightclimb Pathway. Front face {U} via `mana_pool(0,1,0,0,0,0)` is correct.

**Verdict: PASS** (known MDFC limitation)

---

## Card 4: Temple of the False God

**File**: `crates/engine/src/cards/defs/temple_of_the_false_god.rs`
**Session**: F-4 session 6 (conditional mana)

- **Oracle match**: YES
- **Types match**: YES (Land)
- **Mana cost match**: YES (none)
- **DSL correctness**: YES
- **Findings**: None.
  - `mana_pool(0,0,0,0,0,2)` = {C}{C}. Correct.
  - `activation_condition: Some(Condition::ControlAtLeastNOtherLands(4))` models "five or more lands" correctly: Temple itself is a land, so 4 *other* lands + Temple = 5 total. The comment in the file explains this reasoning.

**Verdict: PASS**

---

## Card 5: Bloomvine Regent // Claim Territory

**File**: `crates/engine/src/cards/defs/bloomvine_regent.rs`
**Session**: F-4 (ETB trigger fix)

- **Oracle match**: YES (front face)
- **Types match**: YES (Creature -- Dragon)
- **Mana cost match**: YES ({3}{G}{G})
- **P/T match**: YES (4/5)
- **DSL correctness**: YES
- **Findings**: None.
  - Flying keyword present.
  - Triggered ability uses `WheneverCreatureEntersBattlefield` with `has_subtype: Some(SubType("Dragon"))` and `controller: TargetController::You`. This correctly fires for self (it is a Dragon) and other Dragons you control.
  - Effect `GainLife { player: PlayerTarget::Controller, amount: EffectAmount::Fixed(3) }` matches oracle.

**Verdict: PASS**

---

## Card 6: Den of the Bugbear

**File**: `crates/engine/src/cards/defs/den_of_the_bugbear.rs`
**Session**: F-4 (conditional ETB land)

- **Oracle match**: YES
- **Types match**: YES (Land)
- **Mana cost match**: YES (none)
- **DSL correctness**: YES (implemented portion)
- **Findings**:
  - Conditional ETB tapped: `unless_condition: Some(Condition::Not(Box::new(Condition::ControlAtLeastNOtherLands(2))))` means "enters tapped unless you do NOT control 2+ other lands" which is logically "enters tapped if you control 2+ other lands." Matches oracle.
  - Mana ability `{T}: Add {R}` correct.
  - TODO for land animation ({3}{R}: becomes 3/2 Goblin creature) is a genuine DSL gap (land animation). Valid TODO.

**Verdict: PASS**

---

## Card 7: Oathsworn Vampire

**File**: `crates/engine/src/cards/defs/oathsworn_vampire.rs`
**Session**: F-4 (ETB tapped creature)

- **Oracle match**: YES
- **Types match**: ISSUE -- see F2 below
- **Mana cost match**: YES ({1}{B})
- **P/T match**: YES (2/2)
- **DSL correctness**: YES (implemented portion)
- **Findings**:
  - F2 (MEDIUM / KI-18): Oracle type line is "Creature -- Vampire Knight" but def has `creature_types(&["Knight", "Vampire"])`. The subtype ORDER matters for display but not for game rules. However, the subtypes themselves are both present, so no functional impact. Cosmetic only.
  - ETB tapped replacement: correctly implemented with `EntersTapped`, `is_self: true`.
  - TODO for graveyard casting permission with life-gained condition: valid DSL gap. No stale TODO.

**Verdict: PASS** (cosmetic subtype order issue only)

---

## Card 8: Marang River Regent // Coil and Catch

**File**: `crates/engine/src/cards/defs/marang_river_regent.rs`
**Session**: F-4 (bounce ETB)

- **Oracle match**: YES (front face)
- **Types match**: YES (Creature -- Dragon)
- **Mana cost match**: YES ({4}{U}{U})
- **P/T match**: YES (6/7)
- **DSL correctness**: ISSUE -- see F3/F4 below
- **Findings**:
  - F3 (HIGH / KI-3): TODO says "DSL gap: 'up to N' optional multi-target mechanism." However, the ETB bounce effect (return up to two other target nonland permanents to their owners' hands) CAN be partially expressed. The DSL has `TargetRequirement::TargetPermanentWithFilter(TargetFilter { non_land: true, ... })` for nonland targeting, and multi-target abilities can use multiple `TargetRequirement` entries. The "up to" part IS a genuine gap (you cannot declare fewer targets than the number of TargetRequirements), but the TODO omits the fact that the bounce-to-hand effect and the nonland filter ARE expressible. The TODO should be more specific: "DSL gap: 'up to N' optional targeting (can't declare fewer than N targets)." The current empty `abilities` for the ETB means the creature has no ETB at all, which is a bigger deviation than implementing it with mandatory 2 targets. That said, forced-2-target would also be wrong if only 1 nonland permanent exists. Flagging as HIGH because the TODO is imprecise and a partial implementation may be preferable.
  - F4 (LOW): This is an MDFC (Creature // Instant -- Omen) but the back face (Coil and Catch, {3}{U} instant) is not modeled. Same known MDFC limitation as pathway lands.

**Verdict: FAIL** -- TODO is imprecise (KI-3 adjacent); bounce effect is partially expressible but left completely unimplemented.

---

## Summary

| Card | Verdict | Issues |
|------|---------|--------|
| Shrieking Drake | PASS | Clean |
| Brightclimb Pathway | PASS | Known MDFC limitation (LOW) |
| Clearwater Pathway | PASS | Known MDFC limitation |
| Temple of the False God | PASS | Clean |
| Bloomvine Regent | PASS | Clean |
| Den of the Bugbear | PASS | Valid TODO for land animation |
| Oathsworn Vampire | PASS | Cosmetic subtype order (MEDIUM) |
| Marang River Regent | FAIL | Imprecise TODO; bounce partially expressible (HIGH) |

**Totals**: 7 PASS, 1 FAIL
**Findings**: 1 HIGH, 1 MEDIUM, 2 LOW

### HIGH Issues
- **Marang River Regent** (F3): TODO claims entire ETB is a DSL gap, but bounce-to-hand with nonland filter IS expressible. Only the "up to 2" optional targeting is a true gap. The ETB is left completely unimplemented when a mandatory-2-target version would be closer to correct for most board states.

### MEDIUM Issues
- **Oathsworn Vampire** (F2): Subtype order is "Knight", "Vampire" in def but oracle is "Vampire Knight." Cosmetic; no functional impact.

### LOW Issues
- **Brightclimb Pathway** (F1): MDFC back face not modeled (known systemic limitation).
- **Marang River Regent** (F4): MDFC back face not modeled (known systemic limitation).

### Notes on Missing Files
Five of the eight originally requested cards (Ranger of Eos, Briarbridge Patrol, Spectacle Mage, Jungle Basin, Murkwater Pathway) do not have card definition files in the `defs/` directory. These cards may have been discussed in F-4 review documents but never authored, or may exist under different file names. The Murkwater Pathway case is explained by MDFC naming (filed under `clearwater_pathway.rs`, the front face).
