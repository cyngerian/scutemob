# Ability Plan: CantBeBlocked

**Generated**: 2026-02-26
**CR**: 509.1 (blocking restrictions), 113.12 (not an ability -- quality statement)
**Priority**: P1
**Similar abilities studied**: Flying (`rules/combat.rs:431-438`), Intimidate (`rules/combat.rs:453+`), Equip (`tests/equip.rs`) for activated-ability-with-target pattern

## CR Rule Text

**509.1b** The defending player checks each creature they control to see whether it's
affected by any restrictions (effects that say a creature can't block, or that it can't
block unless some condition is met). If any restrictions are being disobeyed, the
declaration of blockers is illegal.

**113.12** An effect that sets an object's characteristic, or simply states a quality of
that object, is different from an ability granted by an effect. When an object "gains" or
"has" an ability, that ability can be removed by another effect. If an effect defines a
characteristic of the object ("[permanent] is [characteristic value]"), it's not granting
an ability. (See rule 604.3.) Similarly, if an effect states a quality of that object
("[creature] can't be blocked," for example), it's neither granting an ability nor setting
a characteristic.

**509.1a** The defending player chooses which creatures they control, if any, will block.
The chosen creatures must be untapped and they can't also be battles. For each of the
chosen creatures, the defending player chooses one creature for it to block that's
attacking that player, a planeswalker they control, or a battle they protect.

**509.1h** An attacking creature with one or more creatures declared as blockers for it
becomes a blocked creature; one with no creatures declared as blockers for it becomes an
unblocked creature. [...]

## Key Edge Cases

- **CR 113.12**: "can't be blocked" is NOT a keyword ability in the CR. It is a quality
  statement. Per CR 113.12, it is neither granting an ability nor setting a characteristic.
  This means effects like Muraganda Petroglyphs ("creatures with no abilities") still apply
  to a creature that "can't be blocked." The engine models this as a pseudo-keyword
  (`KeywordAbility::CantBeBlocked`) for implementation convenience, which is functionally
  correct for blocking enforcement but technically over-represents it as an "ability."
  This is an acceptable simplification.

- **Rogue's Passage ruling (2018-12-07)**: "Activating the second ability of Rogue's
  Passage after a creature has become blocked won't cause that creature to become
  unblocked." The CantBeBlocked restriction is checked only at declare-blockers time
  (CR 509.1b). Once blocked, the creature stays blocked even if it later gains
  CantBeBlocked. The engine enforces this correctly because the check is in
  `handle_declare_blockers`, not in a continuous re-evaluation.

- **Duration: "this turn"**: Rogue's Passage grants CantBeBlocked until end of turn. The
  effect must expire in cleanup. The engine uses `EffectDuration::UntilEndOfTurn` for this,
  and cleanup step removes such effects (already implemented).

- **Multiplayer**: Any defending player during any combat can encounter the CantBeBlocked
  restriction. The engine's combat code iterates per-blocker-pair, so it naturally handles
  N defenders.

- **Interaction with other evasion**: CantBeBlocked is absolute -- it supersedes flying,
  menace, intimidate, etc. If a creature has CantBeBlocked, no blocker declaration against
  it is ever legal, regardless of what other evasion abilities are present. The engine
  checks CantBeBlocked after flying but before intimidate/landwalk in `combat.rs:441-451`.
  Order does not matter here since all checks must pass.

## Current State (from ability-wip.md)

- [x] Step 1: Enum variant -- `KeywordAbility::CantBeBlocked` at `state/types.rs:172`
- [x] Step 2: Rule enforcement -- blocking restriction at `rules/combat.rs:441-451`
- [x] Step 2b: Hash -- `state/hash.rs:309` (discriminant 24)
- [ ] Step 3: Trigger wiring -- N/A (CantBeBlocked is a restriction, not a trigger)
- [x] Step 4a: Unit test (basic) -- `test_rogues_passage_cant_be_blocked` at `tests/card_def_fixes.rs:572`
- [ ] Step 4b: Unit tests (additional coverage -- see below)
- [x] Step 5: Card definition -- Rogue's Passage at `cards/definitions.rs:400-437`
- [ ] Step 5b: Card definition fix -- Whispersilk Cloak TODO at `definitions.rs:1386`
- [ ] Step 6: Game script

## Implementation Steps

### Step 1: Enum Variant (DONE -- no-op)

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/types.rs`
**Status**: `KeywordAbility::CantBeBlocked` exists at line 172.
**Hash**: Exists at `state/hash.rs:309` (discriminant 24).
**No action needed.**

### Step 2: Rule Enforcement (DONE -- no-op)

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/combat.rs`
**Status**: Blocking restriction enforced at lines 441-451.
**CR**: 509.1b -- "restrictions (effects that say a creature can't block, or that it
can't block unless some condition is met)"
**No action needed.**

### Step 3: Trigger Wiring (N/A)

CantBeBlocked is a blocking restriction, not a trigger. No trigger wiring needed.

### Step 4: Unit Tests

**File**: `/home/airbaggie/scutemob/crates/engine/tests/keywords.rs`
**Existing test**: `test_rogues_passage_cant_be_blocked` in `tests/card_def_fixes.rs:572` --
tests that `DeclareBlockers` is rejected when attacker has `CantBeBlocked`.
**Pattern**: Follow Flying tests at `tests/keywords.rs:241-278` and Equip tests at
`tests/equip.rs:83-153`.

**Tests to write** (add to `tests/keywords.rs` in a new `CantBeBlocked` section):

1. **`test_509_1b_cant_be_blocked_basic`** -- CR 509.1b. Creature with CantBeBlocked
   keyword cannot be declared as blocked. Set up attacker with
   `with_keyword(KeywordAbility::CantBeBlocked)`, blocker without special abilities.
   DeclareBlockers must return Err. (Mirrors the existing test but in keywords.rs with
   the standard pattern.)

2. **`test_509_1b_cant_be_blocked_allows_no_blockers`** -- CR 509.1h. Creature with
   CantBeBlocked, defender declares no blockers (empty blockers vec). DeclareBlockers
   must succeed. Attacker becomes unblocked.

3. **`test_509_1b_cant_be_blocked_other_attacker_can_be_blocked`** -- Two attackers: one
   with CantBeBlocked, one without. Defender blocks only the non-CantBeBlocked creature.
   DeclareBlockers must succeed.

4. **`test_509_1b_cant_be_blocked_plus_flying`** -- Creature with both CantBeBlocked and
   Flying. Blocker with Flying+Reach. Even though the blocker could normally block a
   flyer, CantBeBlocked still prevents it. DeclareBlockers must return Err.

5. **`test_509_1b_cant_be_blocked_via_continuous_effect`** -- Test the full activated-
   ability flow: build a state with an ActivatedAbility that applies
   `ApplyContinuousEffect` granting `AddKeyword(CantBeBlocked)` with
   `UntilEndOfTurn` duration to a target creature. Activate the ability, pass priority
   to resolve, then verify the target creature has `CantBeBlocked` in its calculated
   characteristics. This validates the Rogue's Passage effect pipeline end-to-end
   without depending on the card definition.
   **Pattern**: Follow `test_equip_basic_attaches_to_creature` at `tests/equip.rs:83`.

**Note on test 5**: This is the critical missing test. The existing test only checks the
blocking restriction enforcement directly (keyword pre-set). It does NOT test the activated
ability -> stack -> resolution -> continuous effect -> layer calc -> keyword grant pipeline.
Without this test, a regression in `ApplyContinuousEffect` or `DeclaredTarget` resolution
would go undetected.

### Step 5: Card Definition

**Status**: Rogue's Passage card definition exists at
`/home/airbaggie/scutemob/crates/engine/src/cards/definitions.rs:400-437`.

**Action needed**: Fix the Whispersilk Cloak definition at `definitions.rs:1386-1387`.
The TODO comment says "requires a LayerModification::CantBeBlocked variant (or equivalent
evasion flag) which does not yet exist in the DSL." But `AddKeyword(CantBeBlocked)` DOES
exist and works. Replace the TODO with a `Static` ability definition:

```rust
AbilityDefinition::Static {
    continuous_effect: ContinuousEffectDef {
        layer: EffectLayer::Ability,
        modification: LayerModification::AddKeyword(KeywordAbility::CantBeBlocked),
        filter: EffectFilter::AttachedCreature,
        duration: EffectDuration::WhileSourceOnBattlefield,
    },
},
```

This mirrors the existing Shroud grant at `definitions.rs:1378-1384` for the same card.

**Suggested card for game script**: Rogue's Passage (Land, already defined).
- Oracle text: `{T}: Add {C}. / {4}, {T}: Target creature can't be blocked this turn.`
- Card definition already models both abilities correctly.
- `ability_index: 0` for the CantBeBlocked activated ability (the mana ability `{T}: Add {C}`
  is filtered out by `enrich_spec_from_def` as a mana ability, so the CantBeBlocked ability
  is the only non-mana activated ability at index 0).

### Step 6: Game Script

**Suggested scenario**: Rogue's Passage activation grants CantBeBlocked, then combat
demonstrates the creature cannot be blocked.

**Subsystem directory**: `test-data/generated-scripts/combat/`
**Suggested filename**: `014_rogues_passage_cant_be_blocked.json`
**Suggested id**: `script_combat_014`

**Script outline**:
1. Initial state: P1 has Rogue's Passage + creature on battlefield. P2 has a creature.
   P1 has 5 mana (4 generic + 1 to spare, or 5 colorless). Rogue's Passage untapped.
2. P1 at pre-combat main, activates Rogue's Passage ability (ability_index: 0) targeting
   own creature. Pay {4} + tap Rogue's Passage.
3. All players pass priority. Ability resolves, creature gets CantBeBlocked until EOT.
4. Move to declare attackers. P1 declares the creature as attacker.
5. Move to declare blockers. P2 declares no blockers (because the creature can't be
   blocked -- if P2 tried, it would fail).
6. Move to combat damage. Creature deals damage to P2.
7. Assert: P2 lost life equal to creature's power. Assert: creature is on battlefield.

**Note**: The script must use `activate_ability` action type with `ability_index: 0`,
`card: "Rogue's Passage"`, and `targets: [{ "object": "creature-name" }]`.

### Step 7: Coverage Doc Update

**File**: `/home/airbaggie/scutemob/docs/mtg-engine-ability-coverage.md`
**Action**: Update CantBeBlocked row:
- Status: `validated` (after tests pass and script approved)
- Add test file references
- Add script reference
- Update notes to mention Whispersilk Cloak fix

## Interactions to Watch

- **CantBeBlocked + protection**: Protection's "B" (Blocking) component also prevents
  blocking. If a creature has both CantBeBlocked and Protection from Red, the blocking
  restriction is doubly enforced (irrelevant in practice -- CantBeBlocked is absolute).

- **CantBeBlocked + conditional blocking restrictions** (Menace, Skulk, etc.): CantBeBlocked
  is absolute. Other restrictions are irrelevant when CantBeBlocked is present. The engine
  checks them sequentially and short-circuits at CantBeBlocked.

- **Layer system**: CantBeBlocked via continuous effect (Rogue's Passage, Whispersilk Cloak)
  is applied in Layer 6 (Ability-adding). Effects that remove all abilities (Humility,
  Dress Down) in Layer 6 would remove CantBeBlocked from keywords. Timestamp order matters.

- **"This turn" duration**: The CantBeBlocked effect from Rogue's Passage uses
  `UntilEndOfTurn`. Effects that end the turn early (e.g., Sundial of the Infinite) would
  also end this effect. The engine's cleanup step handles UntilEndOfTurn expiry.

- **Already blocked**: Per Rogue's Passage ruling and CR 509.1, CantBeBlocked only matters
  at declare-blockers time. Gaining CantBeBlocked after blockers are declared does NOT
  unblock the creature.

## Known Gaps (LOW priority, not blocking validation)

- **TargetRequirement on Activated abilities**: The `AbilityDefinition::Activated` variant
  lacks a `targets` field (unlike `Spell`). Rogue's Passage says "Target creature" but the
  engine does not validate that the target is actually a creature. Only hexproof/shroud/
  protection is checked. This is a known LOW-priority gap (general infrastructure, not
  CantBeBlocked-specific). Filed as comment at `abilities.rs:124-126`.

- **CR 113.12 distinction**: The engine models "can't be blocked" as
  `KeywordAbility::CantBeBlocked`, but CR 113.12 says it is neither an ability nor a
  characteristic. This means Muraganda Petroglyphs and similar "no abilities" checks would
  incorrectly see CantBeBlocked as an ability. This is a LOW-priority accuracy gap with
  no practical impact for current card coverage.
