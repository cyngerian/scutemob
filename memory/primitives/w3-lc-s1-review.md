# S1 Audit Review

**Date**: 2026-03-19
**Reviewer**: ability-impl-reviewer (Opus)
**Scope**: Spot-check of W3-LC S1 classifications across 10 files (~110 sites)

---

## HIGH sites verified

### 1. `effects/mod.rs:3730,3747` (PowerOf/ToughnessOf) -- CONFIRMED BUG

The code at line 3730 reads `obj.characteristics.power` and line 3747 reads
`obj.characteristics.toughness`. These are used by `resolve_amount()` which can be
called for any battlefield permanent (e.g., "deals damage equal to its power").
Per CR 613.1/613.4, power and toughness must reflect all layer 7 sublayers including
counters (7c), equipment/Aura grants (7c), anthem effects (7c), and P/T setting (7b).
Reading base characteristics ignores all of these. **Classification correct.**

Note: The audit listed lines as 3712,3729 but the actual `.characteristics.power` read
is at 3730 and `.characteristics.toughness` at 3747. The audit lines point to the
`EffectAmount::PowerOf` match arm start, not the exact read. Minor line-number
imprecision, not a classification error.

### 2. `abilities.rs:6035` (collect_triggers_for_event) -- CONFIRMED BUG

Line 6035 iterates `obj.characteristics.triggered_abilities` for every battlefield
permanent. Per CR 613.1f (Layer 6), ability-removing effects like Humility should
prevent these triggers from existing. Reading base characteristics means Humility
does not suppress triggered abilities. **Classification correct.**

**Additional finding**: Lines 6053-6056 within the same function also read
`entering_obj.characteristics.card_types` for the ETB filter check on another
battlefield permanent. This is a secondary needs-layer-calc site within the same
function that wasn't explicitly listed in the audit table. Fixing 6035 properly
should address this in the same pass.

### 3. `mana.rs:154,157-159,181` (summoning sickness) -- CONFIRMED BUG

Line 154 checks `obj.characteristics.card_types.contains(&CardType::Creature)` and
lines 157-159 check `obj.characteristics.keywords.contains(&KeywordAbility::Haste)`.
Per CR 613.1d (Layer 4) and CR 613.1f (Layer 6), type-changing effects (e.g., animating
an artifact into a creature) and ability-granting effects (e.g., Fervor granting haste)
must be reflected. An animated artifact with summoning sickness would bypass the check
(not seen as a creature), and a creature granted haste by Fervor would still be blocked.
**Classification correct.**

---

## correct-base sites verified

### 1. `abilities.rs:3964` (Myriad trigger post-processing) -- CONFIRMED CORRECT

This reads `obj.characteristics.triggered_abilities.get(t.ability_index)` for a trigger
that was already collected and queued. Per CR 113.7a, once a triggered ability is on the
stack, it exists independently of its source. The read here is to identify the trigger's
description for tagging as `PendingTriggerKind::Myriad`, not to determine whether the
ability exists. The trigger was already collected at line 6035 (which IS the bug site).
**Classification correct.**

Nuance: If 6035 is fixed to use layer-resolved abilities, this post-processing code
would need to index into the same layer-resolved list. But as a classification of
"correct-base for the current code's intent," it holds -- the real fix is upstream.

### 2. `resolution.rs:632` (Suspend creature haste) -- CONFIRMED CORRECT

This reads `obj.characteristics.card_types.contains(&CardType::Creature)` on an object
that was just created by `move_object_to_zone` during spell resolution. The object has
just entered the battlefield and no continuous effects from other permanents have been
evaluated yet in this resolution step. Using base characteristics here is correct --
the check is "was this card printed as a creature" for Suspend's haste grant
(CR 702.62a). Layer-calculated characteristics would also work but are unnecessary
since the object just arrived. **Classification correct.**

### 3. `copy.rs:66` (copy starting point) -- CONFIRMED CORRECT

This reads `obj.characteristics.clone()` as the starting point for copy resolution
(CR 707.2). The copy system explicitly starts from printed/base characteristics and
then applies Layer 1 copy effects on top. Using layer-resolved characteristics here
would be wrong -- it would double-apply layers 2-7. **Classification correct.**

---

## Ambiguous resolutions

### 1. `abilities.rs:3444-3446` (Soulbond pairing) -- RECOMMEND: needs-layer-calc (MEDIUM)

The code checks `obj.characteristics.card_types.contains(&CardType::Creature)` at
3444 for Soulbond pairing candidates. This is a battlefield permanent check. Under
effects like Humility (which doesn't change types, but other effects do -- e.g.,
Song of the Dryads turning a creature into a Forest), a non-creature permanent
shouldn't pair via Soulbond.

The keyword check at 3446 has a partial fix: it falls through to
`calculate_characteristics` at 3448 with an OR. But the card_types check at 3444
has no such fallback -- it uses base types only.

**Recommendation**: `needs-layer-calc` for the card_types check at 3444.
The keyword check at 3446-3454 is already partially layer-aware (the OR-fallback
calculates characteristics), though the short-circuit on base keywords means it
still has a false-positive path (base has Soulbond but layer-resolved doesn't).
Both should use layer-resolved characteristics. Severity: MEDIUM.

### 2. `replacement.rs:1187` (entering-battlefield object) -- RECOMMEND: correct-base

This reads base characteristics for an object that is in the process of entering the
battlefield, specifically to check whether a RemoveAllAbilities effect (like Humility)
applies to it. The comment at 1177 says "We need base characteristics to evaluate
filter predicates."

For an object entering the battlefield, continuous effects haven't fully applied yet.
The function `effect_applies_to_object` uses these characteristics to determine if the
entering permanent matches the filter of a RemoveAllAbilities effect (e.g., "creatures
lose all abilities" -- is this entering object a creature?).

Per CR 614.12, replacement effects that modify how a permanent enters the battlefield
look at the permanent's characteristics as they would exist on the battlefield. However,
for ETB suppression (Humility + ETB triggers), using base characteristics to check "is
this a creature" is defensible because the object hasn't entered yet and Layer 4 type
changes from other permanents shouldn't retroactively change what kind of permanent it
is *before* it arrives. Self-referential replacement effects (like "as ~ enters") are a
different code path.

**Recommendation**: `correct-base`. The base characteristics correctly represent what
the card "is" before entry. Edge case: if the entering object is itself a copy (Clone
entering as a copy of something), the base characteristics might not reflect the copy.
But copy replacement effects are applied earlier in the entry sequence. LOW risk.

### 3. `effects/mod.rs:3798` (CardCount with variable zone) -- RECOMMEND: split

The code at line 3798 calls `matches_filter(&obj.characteristics, f)` for objects
in a zone resolved by `resolve_zone_target`. This zone could be:
- Hand, library, graveyard, exile -- base characteristics are correct
- Battlefield -- needs layer-calculated characteristics

**Recommendation**: `needs-layer-calc` when zone resolves to Battlefield, `correct-base`
otherwise. The fix should check `zone_id` and conditionally calculate characteristics.
Severity: MEDIUM (same as PermanentCount at 3814, which has the identical pattern but
is always battlefield).

---

## Missed sites

### 1. `abilities.rs:6053-6056` -- NOT in audit table

Within `collect_triggers_for_event`, the ETB filter at line 6053 reads
`entering_obj.characteristics.card_types` for the `creature_only` check on a
battlefield permanent. This is a distinct read from the main iteration at 6035.
The audit captured 6035 but not this secondary read within the same function.

**Impact**: MEDIUM. If an animated noncreature (e.g., a land animated by Nissa)
enters the battlefield, the creature_only ETB filter would fail because base
card_types doesn't include Creature. In practice this is rare since most animated
permanents are already on the battlefield, not entering.

### 2. `abilities.rs:4311,4323` (Flanking) -- IN audit but not in summary table

These are listed in the per-file table as "Confirmed bug: Humility breaks it" and
"Confirmed bug: granted Flanking ignored" but are NOT listed in the "Bug Distribution
by Severity" summary at the top. They should be listed as MEDIUM in the summary
(or folded into the HIGH at 6035 since Flanking triggers are collected via
`collect_triggers_for_event` upstream... actually no, the Flanking code at 4311
is separate combat-phase logic, not part of 6035). The reads at 4311/4323 are
standalone battlefield keyword reads.

**Impact**: The summary is incomplete. These are 2 additional MEDIUM sites not
reflected in the severity distribution.

### 3. `resolution.rs:4005,4008` -- Correctly classified as fallback

These are in the "Fallback reads (correct -- skipped)" list at the bottom of the
resolution.rs section. Confirmed: they're fallback for `calculate_characteristics`
at line 3992. No issue.

### 4. No other missed standalone reads found

I checked all `.characteristics.` reads in abilities.rs (40 hits) and resolution.rs
(35 hits) against the audit. All remaining reads are either:
- Already in the audit table
- Listed as fallback patterns
- Mutations (writing to characteristics, not reading for game logic)

---

## Overall assessment

**The audit is trustworthy for S2+ fix sessions.** The classifications are accurate
for all 6 spot-checked sites. Key findings:

1. **All 3 HIGH sites are confirmed genuine bugs.** The severity assignments are
   appropriate -- these affect normal gameplay with common cards (equipment/counters
   for P/T, Humility for triggers, Fervor/animation for summoning sickness).

2. **All 3 correct-base sites are genuinely correct.** The reasoning (CR 113.7a for
   queued triggers, CR 707.2 for copy starting point, just-entered objects) is sound.

3. **Ambiguous sites have clear resolutions**: one is needs-layer-calc (Soulbond types),
   one is correct-base (entering-battlefield Humility check), one needs a conditional
   split (CardCount zone-dependent).

4. **Two minor gaps**: the ETB filter read at 6053-6056 should be added to the S2
   fix list (same function as 6035, fix together), and the Flanking reads at 4311/4323
   should appear in the severity summary.

5. **Line number precision**: a few audit line numbers point to match arms rather than
   the exact `.characteristics.` read. Not a problem for fix sessions since the
   context is clear, but worth noting.

**Recommendation**: Proceed to S2 with confidence. Add abilities.rs:6053-6056 to the
S2 HIGH fix list (same function, same fix). Update the severity summary to include
Flanking (4311/4323) as MEDIUM.
