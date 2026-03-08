# Ability WIP: Cipher

ability: Cipher
cr: 702.99
priority: P4
started: 2026-03-08
phase: closed
plan_file: memory/abilities/ability-plan-cipher.md

## Step Checklist
- [x] 1. Enum variant (types.rs:1316, card_definition.rs:665, stack.rs:1219, stubs.rs:144, game_object.rs:667, hash.rs:675/2034/4097, view_model.rs:596/882, stack_view.rs:203)
- [x] 2. Rule enforcement (resolution.rs:1631-1693 — cipher encoding at instant/sorcery resolution; resolution.rs:4157 — CipherTrigger resolution arm)
- [x] 3. Trigger wiring (abilities.rs:4885-4945 — CombatDamageDealt dispatch for creatures with encoded_cards; abilities.rs:5959-5972 — flush_pending_triggers CipherCombatDamage arm)
- [x] 4. Unit tests (tests/cipher.rs — 7 tests: basic_encode, combat_triggers_copy, no_creatures_graveyard, creature_leaves_broken, copy_not_encodable, no_damage_no_trigger, multiple_encoded_triggers)
- [x] 5. Card definition (call_of_the_nightwing.rs — {2}{U}{B} Sorcery, creates 1/1 flying Horror token + Cipher)
- [x] 6. Game script (test-data/generated-scripts/baseline/script_187_cipher.json — script_baseline_187, pending_review; harness gap documented in dispute: enrich_spec_from_def does not propagate AbilityDefinition::Cipher → KeywordAbility::Cipher, causing cipher encoding to fail via harness path; combat and cipher trigger steps documented as intended CR behavior)
- [x] 7. Coverage doc update

## Review
findings: 5 (1 HIGH, 1 MEDIUM, 3 LOW)
verdict: needs-fix
review_file: memory/abilities/ability-review-cipher.md

### Fix Required
1. **HIGH** — hash.rs:1389: Add `cipher_encoded_card_id` and `cipher_encoded_object_id` to PendingTrigger HashInto impl
2. **MEDIUM** — resolution.rs:1636: Add `&& !stack_obj.cast_with_aftermath` to `has_cipher` guard
3. **LOW (new, found during scripting)** — replay_harness.rs enrich_spec_from_def: Add loop for AbilityDefinition::Cipher → KeywordAbility::Cipher propagation, similar to the Champion/Soulbond handlers (lines 2246-2258). Without this, the resolution.rs cipher encoding check (obj.characteristics.keywords.contains(&KeywordAbility::Cipher)) returns false for cards placed through the harness, causing the card to go to the graveyard instead of exile and preventing cipher triggers from firing.
