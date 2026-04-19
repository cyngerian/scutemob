// Balefire Dragon — {5}{R}{R}, Creature — Dragon 6/6
// Flying
// Whenever this creature deals combat damage to a player, it deals that much damage to
// each creature that player controls.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("balefire-dragon"),
        name: "Balefire Dragon".to_string(),
        mana_cost: Some(ManaCost { generic: 5, red: 2, ..Default::default() }),
        types: creature_types(&["Dragon"]),
        oracle_text: "Flying\nWhenever this creature deals combat damage to a player, it deals that much damage to each creature that player controls.".to_string(),
        power: Some(6),
        toughness: Some(6),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            // CR 510.3a: "Whenever this creature deals combat damage to a player, it deals that
            // much damage to each creature that player controls."
            // CombatDamageDealt resolves from ctx.combat_damage_amount (propagated through ForEach
            // inner context at effects/mod.rs:2419). DamagedPlayer scopes the ForEach to creatures
            // controlled by the specific player dealt damage.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenDealsCombatDamageToPlayer,
                effect: Effect::ForEach {
                    over: ForEachTarget::EachPermanentMatching(Box::new(TargetFilter {
                        has_card_type: Some(CardType::Creature),
                        controller: TargetController::DamagedPlayer,
                        ..Default::default()
                    })),
                    effect: Box::new(Effect::DealDamage {
                        target: EffectTarget::DeclaredTarget { index: 0 },
                        amount: EffectAmount::CombatDamageDealt,
                    }),
                },
                intervening_if: None,
                targets: vec![],
                modes: None,
                trigger_zone: None,
            },
        ],
        ..Default::default()
    }
}
