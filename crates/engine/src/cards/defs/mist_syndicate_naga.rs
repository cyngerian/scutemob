// Mist-Syndicate Naga — {2}{U}, Creature — Snake Ninja 3/1
// Ninjutsu {2}{U}
// Whenever this creature deals combat damage to a player, create a token that's a copy of
// this creature.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("mist-syndicate-naga"),
        name: "Mist-Syndicate Naga".to_string(),
        mana_cost: Some(ManaCost { generic: 2, blue: 1, ..Default::default() }),
        types: creature_types(&["Snake", "Ninja"]),
        oracle_text: "Ninjutsu {2}{U} ({2}{U}, Return an unblocked attacker you control to hand: Put this card onto the battlefield from your hand tapped and attacking.)\nWhenever this creature deals combat damage to a player, create a token that's a copy of this creature.".to_string(),
        power: Some(3),
        toughness: Some(1),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Ninjutsu),
            AbilityDefinition::Ninjutsu {
                cost: ManaCost { generic: 2, blue: 1, ..Default::default() },
            },
            // Whenever this creature deals combat damage to a player, create a token
            // copy of this creature.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenDealsCombatDamageToPlayer,
                effect: Effect::CreateTokenCopy {
                    source: EffectTarget::Source,
                    enters_tapped_and_attacking: false,
                },
                intervening_if: None,
                targets: vec![],
            },
        ],
        ..Default::default()
    }
}
