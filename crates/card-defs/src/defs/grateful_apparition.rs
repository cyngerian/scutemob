// Grateful Apparition — {1}{W}, Creature — Spirit 1/1
// Flying
// Whenever this creature deals combat damage to a player or planeswalker, proliferate.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("grateful-apparition"),
        name: "Grateful Apparition".to_string(),
        mana_cost: Some(ManaCost {
            generic: 1,
            white: 1,
            ..Default::default()
        }),
        types: creature_types(&["Spirit"]),
        oracle_text: "Flying\nWhenever this creature deals combat damage to a player or \
                      planeswalker, proliferate. (Choose any number of permanents and/or players, \
                      then give each another counter of each kind already there.)"
            .to_string(),
        power: Some(1),
        toughness: Some(1),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            AbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition: TriggerCondition::WhenDealsCombatDamageToPlayer,
                effect: Effect::Proliferate,
                intervening_if: None,
                targets: vec![],

                modes: None,
                trigger_zone: None,
            },
        ],
        // PB-DX4 (2026-08-01, OOS-DP10-8): Complete (by the `#[default]` derive) -> partial.
        //
        // MCP-verified printed text: "Whenever this creature deals combat damage to a player
        // OR PLANESWALKER, proliferate." The trigger above is
        // `TriggerCondition::WhenDealsCombatDamageToPlayer`, and its only dispatch site
        // (`rules/abilities.rs`, the combat-damage arm) is gated on
        // `matches!(assignment.target, CombatDamageTarget::Player(_))` -- so connecting with a
        // planeswalker never proliferates. The def's own stored `oracle_text` says "or
        // planeswalker", so it already contradicted its own encoding.
        //
        // Not expressible today: `TriggerCondition` has a self "deals combat damage to a
        // player" variant and an EQUIPPED-creature "deals combat damage (any recipient)"
        // variant (`WhenEquippedCreatureDealsCombatDamage`), but no self any-recipient
        // variant. Adding one is an engine + wire change, out of scope for this
        // card-def-only batch.
        completeness: Completeness::partial(
            "Printed 'deals combat damage to a player or planeswalker'; \
             TriggerCondition::WhenDealsCombatDamageToPlayer fires only on damage to a player \
             (rules/abilities.rs gates on CombatDamageTarget::Player), so combat damage to a \
             planeswalker does not proliferate. No self any-recipient combat-damage \
             TriggerCondition exists yet (only the equipped-creature counterpart).",
        ),
        ..Default::default()
    }
}
