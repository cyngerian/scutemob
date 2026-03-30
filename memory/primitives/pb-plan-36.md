# Primitive Batch Plan: PB-36 -- Evasion/Protection Extensions

**Generated**: 2026-03-29
**Primitive**: CantBlock keyword, CantBeBlockedExceptBy filtered evasion, player protection effect, ProtectionQuality extensions
**CR Rules**: 702.111 (Menace), 702.16 (Protection), 509.1b (Blocking restrictions)
**Cards affected**: ~21 (18 existing fixes + 3 wiring-only fixes)
**Dependencies**: None (all prerequisite PBs complete)
**Deferred items from prior PBs**: None specific to evasion/protection

## Primitive Specification

PB-36 adds four engine capabilities to the DSL:

1. **`KeywordAbility::CantBlock`** -- A keyword representing "this creature can't block." Currently only Decayed (702.147) and Suspected (701.60c) provide this restriction via ad-hoc checks in combat.rs. Many cards (Bloodghast, Carrion Feeder, Phoenix Chick, Skrelv, Phyrexian Mite tokens) have "can't block" as a static ability unrelated to Decayed/Suspected. This keyword is enforced identically: during blocker declaration, creatures with CantBlock are rejected.

2. **`KeywordAbility::CantBeBlockedExceptBy(BlockingExceptionFilter)`** -- A parameterized evasion keyword for "can't be blocked except by creatures with [quality]." This covers Signal Pest ("except by creatures with flying or reach"), Gingerbrute ("except by creatures with haste"), and similar cards. The existing `CantBeBlocked` keyword has no filter. The new variant carries a filter enum describing what blockers are permitted.

3. **`Effect::GrantPlayerProtection`** -- An effect that adds `ProtectionQuality` entries to `PlayerState.protection_qualities`. The infrastructure for player protection (targeting check in casting.rs, damage prevention in replacement.rs, hashing) already exists but there is no Effect variant to populate it from card defs. This unblocks Teferi's Protection and The One Ring.

4. **Wiring-only protection fixes** -- Several cards (Emrakul, Greensleeves, Sword of Body and Mind) can use the existing `ProtectionFrom(ProtectionQuality)` keyword but their card defs were authored before the ProtectionQuality enum existed. These are pure card-def wiring fixes with no engine change.

### Out of Scope (deferred to PB-37)

- **Color choice at activation time** (Mother of Runes, Alseid of Life's Bounty, Commander's Plate) -- requires interactive player choice infrastructure
- **Dynamic power-based blocking restrictions** (Champion of Lambholt, Den Protector) -- "creatures with power less than X can't block" requires a continuous-effect blocking restriction evaluated dynamically at block declaration
- **Mass blocking restrictions as spell effects** (Sundering Eruption "creatures without flying can't block this turn") -- requires a GameRestriction-level blocking filter with duration
- **Attack restrictions** (Lovestruck Beast "can't attack unless") -- ContinuousRestriction extension, not evasion
- **Keyword counters** (Biting-Palm Ninja menace counter) -- requires keyword-counter infrastructure (CR 122.1b)
- **Hexproof from abilities** (Volatile Stormdrake) -- needs a HexproofFrom variant
- **Complex multi-gap cards** (Akroma's Will, Kaito Shizuki, Vampire Gourmand, Seasoned Dungeoneer) -- these have multiple unrelated DSL gaps beyond just evasion/protection

## CR Rule Text

### CR 702.111 -- Menace
> 702.111a Menace is an evasion ability.
> 702.111b A creature with menace can't be blocked except by two or more creatures. (See rule 509, "Declare Blockers Step.")
> 702.111c Multiple instances of menace on the same creature are redundant.

### CR 702.16 -- Protection
> 702.16a Protection is a static ability, written "Protection from [quality]." This quality is usually a color but can be any characteristic value or information.
> 702.16b A permanent or player with protection can't be targeted by spells with the stated quality and can't be targeted by abilities from a source with the stated quality.
> 702.16e Any damage that would be dealt by sources that have the stated quality to a permanent or player with protection is prevented.
> 702.16f Attacking creatures with protection can't be blocked by creatures that have the stated quality.
> 702.16j "Protection from everything" is a variant of the protection ability. A permanent or player with protection from everything has protection from each object regardless of that object's characteristic values.
> 702.16k "Protection from [a player]" is a variant of the protection ability. A permanent or player with protection from a specific player has protection from each object that player controls and protection from each object that player owns not controlled by another player.

### CR 509.1b -- Blocking Restrictions
> 509.1b The defending player checks each creature they control to see whether it's affected by any restrictions (effects that say a creature can't block, or that it can't block unless some condition is met). If any restrictions are being disobeyed, the declaration of blockers is illegal.

## Engine Changes

### Change 1: Add `KeywordAbility::CantBlock` variant

**File**: `crates/engine/src/state/types.rs`
**Action**: Add `CantBlock` variant to `KeywordAbility` enum after `CantBeBlocked` (line ~370)
**Pattern**: Follow `CantBeBlocked` at line 370

```
/// CR 509.1b: This creature can't block. Static restriction enforced in combat.rs
/// blocker declaration. Unlike Decayed (which also prevents blocking), this is a
/// standalone keyword for cards like Bloodghast, Carrion Feeder, Phoenix Chick.
CantBlock,
```

### Change 2: Add `BlockingExceptionFilter` enum and `CantBeBlockedExceptBy` variant

**File**: `crates/engine/src/state/types.rs`
**Action**: Add new enum `BlockingExceptionFilter` near `ProtectionQuality` (line ~87), then add `CantBeBlockedExceptBy(BlockingExceptionFilter)` variant to `KeywordAbility`

```rust
/// Filter describing which creatures are permitted to block a creature with
/// CantBeBlockedExceptBy. Used for cards like Signal Pest and Gingerbrute.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum BlockingExceptionFilter {
    /// "except by creatures with [keyword]" -- e.g., flying, reach, haste
    HasKeyword(KeywordAbility),
    /// "except by creatures with [keyword] and/or [keyword]" -- e.g., flying and/or reach
    HasAnyKeyword(Vec<KeywordAbility>),
}
```

And in `KeywordAbility`:
```
/// CR 509.1b: "This creature can't be blocked except by creatures with [quality]."
/// Parameterized evasion -- the filter specifies which blockers are permitted.
/// Example: Signal Pest has CantBeBlockedExceptBy(HasAnyKeyword(vec![Flying, Reach]))
CantBeBlockedExceptBy(BlockingExceptionFilter),
```

### Change 3: Enforce CantBlock in combat.rs blocker declaration

**File**: `crates/engine/src/rules/combat.rs`
**Action**: Add a check for `KeywordAbility::CantBlock` in the blocker validation section, after the Decayed check (~line 612) and Suspected check (~line 619). Also add to the requirement-impossibility checks (~line 918).

**CR**: 509.1b -- blocking restriction enforcement

In `handle_declare_blockers` per-blocker validation (around line 612):
```rust
// CR 509.1b: A creature with CantBlock can't block.
if blocker_chars.keywords.contains(&KeywordAbility::CantBlock) {
    return Err(GameStateError::InvalidCommand(format!(
        "Object {:?} has CantBlock and cannot block (CR 509.1b)",
        blocker_id
    )));
}
```

In the Provoke requirement-impossibility section (around line 918):
```rust
// CR 509.1b: CantBlock creatures can't block.
if provoked_chars.keywords.contains(&KeywordAbility::CantBlock) {
    continue; // Requirement impossible -- skip
}
```

### Change 4: Enforce CantBeBlockedExceptBy in combat.rs

**File**: `crates/engine/src/rules/combat.rs`
**Action**: Add a check for `CantBeBlockedExceptBy` in the per-attacker blocking validation section, after the CantBeBlocked check (~line 690) and before/after the Intimidate check (~line 701). Also add to the requirement-impossibility section (~line 926) and the menace check section (~line 838).

**CR**: 509.1b -- blocking restriction enforcement

```rust
// CR 509.1b: CantBeBlockedExceptBy -- creature can only be blocked by creatures
// matching the exception filter.
for kw in attacker_chars.keywords.iter() {
    if let KeywordAbility::CantBeBlockedExceptBy(filter) = kw {
        let blocker_matches = match filter {
            BlockingExceptionFilter::HasKeyword(required_kw) => {
                blocker_chars.keywords.contains(required_kw)
            }
            BlockingExceptionFilter::HasAnyKeyword(required_kws) => {
                required_kws.iter().any(|k| blocker_chars.keywords.contains(k))
            }
        };
        if !blocker_matches {
            return Err(GameStateError::InvalidCommand(format!(
                "Object {:?} cannot block {:?} (attacker has CantBeBlockedExceptBy; \
                 blocker does not match filter {:?})",
                blocker_id, attacker_id, filter
            )));
        }
    }
}
```

### Change 5: Add `Effect::GrantPlayerProtection` variant

**File**: `crates/engine/src/cards/card_definition.rs`
**Action**: Add new variant to the `Effect` enum

```rust
/// CR 702.16b/e/j: Grant protection qualities to a player.
/// Populates `PlayerState.protection_qualities` with the given qualities.
/// Used by Teferi's Protection ("you gain protection from everything") and
/// The One Ring ("you gain protection from everything until your next turn").
GrantPlayerProtection {
    player: PlayerTarget,
    qualities: Vec<ProtectionQuality>,
},
```

Note: Duration is NOT stored on the Effect itself. Instead, the runner must pair this with a `PlayerProtectionExpiration` entry on `GameState` (see Change 5a). The Effect simply pushes qualities; the expiration mechanism handles cleanup.

### Change 5a: Add `EffectDuration::UntilYourNextTurn` and player protection expiration

**File**: `crates/engine/src/state/continuous_effect.rs`
**Action**: Add `UntilYourNextTurn` variant to `EffectDuration` enum (after `Indefinite`, line ~52)

```rust
/// Expires at the beginning of the controller's next turn (before untap).
/// Used for "until your next turn" effects like Teferi's Protection and
/// The One Ring's ETB protection grant.
UntilYourNextTurn,
```

**File**: `crates/engine/src/state/player.rs`
**Action**: Add a `player_protection_expiration` field to track when temporary protection expires. Alternative: add a `Vec<(Vec<ProtectionQuality>, ExpirationInfo)>` structure. Simplest approach:

```rust
/// Turn number at which temporary protection qualities expire.
/// When the current turn reaches this value AND the active player matches,
/// the protection_qualities vec is cleared.
/// None = no expiration (permanent protection or no temporary protection active).
#[serde(default)]
pub player_protection_expires_turn: Option<u32>,
```

**File**: `crates/engine/src/rules/turn_actions.rs` (or wherever turn start is processed)
**Action**: At the beginning of a player's turn (untap step or pre-untap), check if `player_protection_expires_turn == Some(current_turn_number)` and clear `protection_qualities` + reset the expiration field.

**Alternative (simpler)**: Skip the expiration mechanism entirely for PB-36. Instead, just add `GrantPlayerProtection` as an effect that pushes qualities permanently. The "until your next turn" cleanup can be a follow-up. This lets us wire The One Ring and Teferi's Protection partially (protection is granted but never expires). Document this as a known limitation.

**Recommendation**: Use the simpler approach. Add `GrantPlayerProtection` without duration, mark expiration as TODO. The protection infrastructure (targeting + damage prevention) already works; the expiration is a separate cleanup concern. This keeps PB-36 to 1 session.

### Change 6: Dispatch GrantPlayerProtection in effects/mod.rs

**File**: `crates/engine/src/effects/mod.rs`
**Action**: Add match arm in the effect execution function for `Effect::GrantPlayerProtection`.

```rust
Effect::GrantPlayerProtection { player, qualities } => {
    let player_id = resolve_player_target(&player, &context, state)?;
    if let Some(ps) = state.players.get_mut(&player_id) {
        for q in qualities {
            ps.protection_qualities.push(q.clone());
        }
    }
    Ok(vec![])
}
```

**CR**: 702.16j -- protection from everything

### Change 7: Hash new KeywordAbility variants

**File**: `crates/engine/src/state/hash.rs`
**Action**: Add hash arms for `CantBlock` (discriminant 160) and `CantBeBlockedExceptBy` (discriminant 161)

```rust
KeywordAbility::CantBlock => 160u8.hash_into(hasher),
KeywordAbility::CantBeBlockedExceptBy(filter) => {
    161u8.hash_into(hasher);
    filter.hash_into(hasher);
},
```

Also add `HashInto` impl for `BlockingExceptionFilter`:
```rust
impl HashInto for BlockingExceptionFilter {
    fn hash_into(&self, hasher: &mut Hasher) {
        match self {
            BlockingExceptionFilter::HasKeyword(kw) => {
                0u8.hash_into(hasher);
                kw.hash_into(hasher);
            }
            BlockingExceptionFilter::HasAnyKeyword(kws) => {
                1u8.hash_into(hasher);
                for kw in kws {
                    kw.hash_into(hasher);
                }
            }
        }
    }
}
```

### Change 8: Hash GrantPlayerProtection effect

**File**: `crates/engine/src/state/hash.rs`
**Action**: Add hash arm for `Effect::GrantPlayerProtection` in the Effect HashInto match. Use the next available Effect discriminant.

### Change 9: Exhaustive match updates

Files requiring new match arms for `CantBlock` and `CantBeBlockedExceptBy`:

| File | Match expression | Action |
|------|-----------------|--------|
| `crates/engine/src/state/hash.rs` | KeywordAbility HashInto | Add disc 160 (CantBlock), disc 161 (CantBeBlockedExceptBy) |
| `crates/engine/src/state/hash.rs` | BlockingExceptionFilter | New HashInto impl |
| `crates/engine/src/state/hash.rs` | Effect HashInto | Add arm for GrantPlayerProtection |
| `crates/engine/src/state/hash.rs` | EffectDuration HashInto | Add arm for UntilYourNextTurn (if added) |
| `tools/replay-viewer/src/view_model.rs` | KeywordAbility display | Add `CantBlock => "Can't Block"`, `CantBeBlockedExceptBy(_) => "Evasion (filtered)"` |
| `crates/engine/src/rules/combat.rs` | Blocker validation | Add CantBlock check (Change 3), CantBeBlockedExceptBy check (Change 4) |
| `crates/engine/src/rules/combat.rs` | Provoke requirement skip | Add CantBlock impossibility check |
| `crates/engine/src/rules/combat.rs` | legal_actions blocker gen | Skip CantBlock creatures (if there is blocker legality filtering) |
| `crates/engine/src/effects/mod.rs` | Effect dispatch | Add GrantPlayerProtection arm |

Note: Check if there are other exhaustive matches on `KeywordAbility` (the `_` wildcard arm in some matches may cover new variants implicitly, but explicit is better). Also check `crates/engine/src/cards/helpers.rs` -- if `BlockingExceptionFilter` needs to be re-exported for card defs to use it.

## Card Definition Fixes

### Group A: CantBlock keyword (5 cards + 3 token fixes)

#### bloodghast.rs
**Oracle text**: "This creature can't block. This creature has haste as long as an opponent has 10 or less life. Landfall -- Whenever a land you control enters, you may return this card from your graveyard to the battlefield."
**Current state**: TODO for "can't block" -- `KeywordAbility::CantBlock` does not exist
**Fix**: Add `AbilityDefinition::Keyword(KeywordAbility::CantBlock)` to abilities vec

#### carrion_feeder.rs
**Oracle text**: "Carrion Feeder can't block. Sacrifice a creature: Put a +1/+1 counter on Carrion Feeder."
**Current state**: TODO for "can't block"
**Fix**: Add `AbilityDefinition::Keyword(KeywordAbility::CantBlock)` at start of abilities vec

#### phoenix_chick.rs
**Oracle text**: "Flying, haste. This creature can't block. Whenever you attack with three or more creatures, you may pay {R}{R}. If you do, return this card from your graveyard to the battlefield tapped and attacking with a +1/+1 counter on it."
**Current state**: TODO for "can't block"
**Fix**: Add `AbilityDefinition::Keyword(KeywordAbility::CantBlock)` after Flying + Haste keywords

#### skrelv_defector_mite.rs
**Oracle text**: "Toxic 1. Skrelv can't block. {W/P}, {T}: Choose a color..."
**Current state**: TODO for "can't block" (activated ability remains a separate gap)
**Fix**: Add `AbilityDefinition::Keyword(KeywordAbility::CantBlock)` after Toxic keyword

#### vishgraz_the_doomhive.rs
**Oracle text**: Tokens have "This token can't block"
**Current state**: Comment says CantBlock omitted
**Fix**: Add `KeywordAbility::CantBlock` to token's `keywords` OrdSet

#### skrevls_hive.rs
**Oracle text**: Creates tokens with "This token can't block"
**Current state**: Comment says can't block restriction missing from tokens
**Fix**: Add `KeywordAbility::CantBlock` to token's `keywords` OrdSet

#### white_suns_twilight.rs
**Oracle text**: Creates tokens with "This token can't block"
**Current state**: Comment notes CantBlock DSL gap
**Fix**: Add `KeywordAbility::CantBlock` to token's `keywords` OrdSet

### Group B: CantBeBlockedExceptBy filtered evasion (2 cards)

#### signal_pest.rs
**Oracle text**: "This creature can't be blocked except by creatures with flying or reach."
**Current state**: TODO for blocking restriction
**Fix**: Add `AbilityDefinition::Keyword(KeywordAbility::CantBeBlockedExceptBy(BlockingExceptionFilter::HasAnyKeyword(vec![KeywordAbility::Flying, KeywordAbility::Reach])))`

#### gingerbrute.rs
**Oracle text**: "{1}: This creature can't be blocked this turn except by creatures with haste."
**Current state**: TODO for filtered evasion
**Fix**: Add activated ability with `Cost::Mana(ManaCost { generic: 1, ..Default::default() })` that applies `CantBeBlockedExceptBy(HasKeyword(KeywordAbility::Haste))` as a continuous effect until end of turn via `ApplyContinuousEffect` with `LayerModification::AddKeyword(KeywordAbility::CantBeBlockedExceptBy(...))` and `EffectDuration::UntilEndOfTurn`

### Group C: Protection wiring fixes (6 cards, no new engine code)

#### emrakul_the_promised_end.rs
**Oracle text**: "Flying, trample, protection from instants"
**Current state**: TODO says "Protection(filter) for instant card type not in DSL"
**Fix**: Add `AbilityDefinition::Keyword(KeywordAbility::ProtectionFrom(ProtectionQuality::FromCardType(CardType::Instant)))` -- this already works, the card def author just didn't use it

#### greensleeves_maro_sorcerer.rs
**Oracle text**: "Protection from planeswalkers and from Wizards"
**Current state**: TODO says "multi-quality protection not expressible"
**Fix**: Add two keywords:
- `AbilityDefinition::Keyword(KeywordAbility::ProtectionFrom(ProtectionQuality::FromCardType(CardType::Planeswalker)))`
- `AbilityDefinition::Keyword(KeywordAbility::ProtectionFrom(ProtectionQuality::FromSubType(SubType("Wizard".to_string()))))`

#### sword_of_body_and_mind.rs
**Oracle text**: "Equipped creature gets +2/+2 and has protection from green and from blue."
**Current state**: TODO for protection grant on equipped creature
**Fix**: Add two Static abilities with `EffectFilter::AttachedCreature`, `EffectLayer::Ability`, `LayerModification::AddKeyword(KeywordAbility::ProtectionFrom(ProtectionQuality::FromColor(Color::Green)))` and same for Blue. Duration: `WhileSourceOnBattlefield`.

#### cryptic_coat.rs
**Oracle text**: "Equipped creature gets +1/+0 and can't be blocked."
**Current state**: TODO for static grant -- "+1/+0 and can't be blocked"
**Fix**: Add two Static abilities:
1. `EffectLayer::PtModify`, `LayerModification::ModifyPower(1)`, `EffectFilter::AttachedCreature`
2. `EffectLayer::Ability`, `LayerModification::AddKeyword(KeywordAbility::CantBeBlocked)`, `EffectFilter::AttachedCreature`

Note: This uses the existing `CantBeBlocked` keyword (not the new filtered variant) since the oracle says "can't be blocked" with no exception.

#### untimely_malfunction.rs
**Oracle text**: Mode 2: "One or two target creatures can't block this turn."
**Current state**: TODO for CantBlock-until-EOT effect
**Fix**: Mode 2 effect becomes `ApplyContinuousEffect` granting `CantBlock` keyword to target(s) until end of turn. This uses the new `CantBlock` keyword with `LayerModification::AddKeyword(KeywordAbility::CantBlock)` and `EffectDuration::UntilEndOfTurn`.

Note: Mode 1 (counter target spell or ability) remains a separate DSL gap.

#### quilled_charger.rs
**Oracle text**: "Whenever this creature attacks while saddled, it gets +1/+2 and gains menace until end of turn."
**Current state**: TODO for "gains menace until end of turn" during saddled attack
**Fix**: The menace grant portion can now be expressed (Menace keyword already exists). The attack-while-saddled trigger is a separate gap, but the menace grant itself is wiring-only. If the trigger already exists, add `ApplyContinuousEffect` with `AddKeyword(Menace)` + `UntilEndOfTurn`. Check current state of the card def at implementation time.

### Group D: Player protection effect (2 cards, partial fix)

#### teferis_protection.rs
**Oracle text**: "Until your next turn, your life total can't change and you gain protection from everything. All permanents you control phase out. Exile Teferi's Protection."
**Current state**: All TODOs -- empty abilities
**Fix (partial)**: Add spell ability with:
- `Effect::GrantPlayerProtection { player: Controller, qualities: vec![ProtectionQuality::FromAll] }`
- `Effect::ExileObject { target: Source }` for self-exile (if ExileObject supports EffectTarget::Source)

Note: "life total can't change" is a separate prevention effect (out of scope). Phase-out of all permanents may need `Effect::PhaseOut` for all controller permanents (check if this exists). The player protection part is the PB-36 fix. Duration cleanup (until your next turn) deferred -- protection is granted permanently until cleanup infrastructure is added.

#### the_one_ring.rs
**Oracle text**: "When The One Ring enters, if you cast it, you gain protection from everything until your next turn."
**Current state**: TODO for ETB protection grant
**Fix (partial)**: Add triggered ability for ETB-if-cast:
- `TriggerCondition::WhenEntersBattlefield` with `intervening_if: Some(Condition::WasCast)` (if Condition::WasCast exists)
- `Effect::GrantPlayerProtection { player: Controller, qualities: vec![ProtectionQuality::FromAll] }`

Note: Other TODOs (burden counter draw scaling, upkeep life loss scaling) remain separate gaps. Duration cleanup deferred.

## New Card Definitions

None -- all ~21 affected cards already have card def files.

## Unit Tests

**File**: `crates/engine/tests/evasion_protection.rs` (new file)
**Tests to write**:

- `test_cant_block_keyword_prevents_blocking` -- CR 509.1b: creature with CantBlock is rejected as blocker
- `test_cant_block_keyword_does_not_prevent_attacking` -- CantBlock creatures can still attack
- `test_cant_block_keyword_skip_provoke_requirement` -- Provoked creature with CantBlock: requirement impossible, skip
- `test_cant_be_blocked_except_by_keyword_allows_matching` -- Signal Pest: creature with flying CAN block it
- `test_cant_be_blocked_except_by_keyword_rejects_nonmatching` -- Signal Pest: creature without flying/reach CANNOT block it
- `test_cant_be_blocked_except_by_has_any_keyword` -- CantBeBlockedExceptBy with two keywords (flying OR reach)
- `test_cant_be_blocked_except_by_combined_with_menace` -- creature with both menace and CantBeBlockedExceptBy: must satisfy both
- `test_grant_player_protection_prevents_targeting` -- CR 702.16b/j: player with protection from everything can't be targeted
- `test_grant_player_protection_prevents_damage` -- CR 702.16e/j: damage to protected player is prevented
- `test_protection_from_card_type_blocks_instants` -- CR 702.16a: Emrakul with protection from instants
- `test_protection_from_subtype_blocks_wizards` -- CR 702.16a: protection from Wizards

**Pattern**: Follow tests in `crates/engine/tests/keywords.rs` (e.g., test_skulk_* at lines ~768+, test_hexproof_player_* at lines ~514+)

## Verification Checklist

- [ ] Engine primitive compiles (`cargo check`)
- [ ] All existing card def TODOs for this batch resolved
- [ ] New card defs authored (if any)
- [ ] Unit tests pass (`cargo test --all`)
- [ ] Clippy clean (`cargo clippy -- -D warnings`)
- [ ] Workspace builds (`cargo build --workspace`)
- [ ] No remaining TODOs in affected card defs (for PB-36 scope)

## Session Estimate

**1 session** (engine changes + card fixes + tests). The engine changes are small and focused:
- 2 new KeywordAbility variants (CantBlock, CantBeBlockedExceptBy)
- 1 new enum (BlockingExceptionFilter)
- 1 new Effect variant (GrantPlayerProtection)
- Combat.rs enforcement (3-4 insertion points)
- Hash updates
- ~15 card def fixes (mostly adding keywords)
- 11 unit tests

## Risks & Edge Cases

- **CantBlock + Provoke interaction**: A creature with CantBlock that is provoked has an impossible requirement. CR 509.1c says the maximum number of obeyed requirements is computed -- if CantBlock makes blocking impossible, the requirement is skipped. This is already handled by the requirement-impossibility check section in combat.rs; we just need to add CantBlock to the skip conditions.

- **CantBeBlockedExceptBy + Menace stacking**: If a creature has both Menace and CantBeBlockedExceptBy(Flying/Reach), it must be blocked by 2+ creatures AND all blockers must have flying or reach. Both constraints apply independently. The existing menace check (line ~838) fires after per-blocker validation, so this works naturally.

- **CantBeBlockedExceptBy + Protection stacking**: Protection from a quality also prevents blocking. If a creature has both CantBeBlockedExceptBy and ProtectionFrom, BOTH must be satisfied. This works because the checks are independent `return Err` blocks.

- **GrantPlayerProtection duration cleanup**: `UntilYourNextTurn` does NOT exist in `EffectDuration` yet. The existing durations are: `WhileSourceOnBattlefield`, `UntilEndOfTurn`, `Indefinite`, `WhilePaired`. For PB-36, use the simpler approach: grant protection without duration tracking. Mark expiration as a known limitation with a TODO. The targeting/damage-prevention infrastructure works correctly regardless of duration tracking.

- **GrantPlayerProtection and "life total can't change"**: This is a separate effect on Teferi's Protection that should NOT be conflated with protection. Protection from everything prevents damage, but "life total can't change" also prevents life gain and non-damage life loss. The plan correctly separates these.

- **TokenSpec with CantBlock**: Tokens created with `CantBlock` in their `keywords` OrdSet will have this keyword through the layer system like any other keyword. It can be removed by Humility/similar effects. This is correct behavior.

- **BlockingExceptionFilter recursive type**: `BlockingExceptionFilter::HasKeyword(KeywordAbility)` contains a `KeywordAbility`, which could in theory contain a `CantBeBlockedExceptBy(BlockingExceptionFilter)` -- creating a recursive type. This is fine because `KeywordAbility` is an enum (not recursive by default) and `BlockingExceptionFilter` is `Ord`/`Hash`/`Eq` derived. No `Box` needed since the nesting is only 1 level deep in practice.

- **Replay viewer exhaustive match**: The runner must add display arms for CantBlock and CantBeBlockedExceptBy in `tools/replay-viewer/src/view_model.rs` or the workspace build will fail.

- **helpers.rs re-export**: If `BlockingExceptionFilter` is needed in card defs (Signal Pest, Gingerbrute), it must be re-exported from `crates/engine/src/cards/helpers.rs`. Check if `KeywordAbility` variants with payloads need their payload types exported there.
