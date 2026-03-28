// Professional Face-Breaker — {2}{R}, Creature — Human Warrior 2/3
// Menace
// Whenever one or more creatures you control deal combat damage to a player, create a
// Treasure token.
// Sacrifice a Treasure: Exile the top card of your library. You may play that card this turn.
//
// TODO: "Sacrifice a Treasure: exile top card + impulse draw" not expressible.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("professional-face-breaker"),
        name: "Professional Face-Breaker".to_string(),
        mana_cost: Some(ManaCost { generic: 2, red: 1, ..Default::default() }),
        types: creature_types(&["Human", "Warrior"]),
        oracle_text: "Menace\nWhenever one or more creatures you control deal combat damage to a player, create a Treasure token.\nSacrifice a Treasure: Exile the top card of your library. You may play that card this turn.".to_string(),
        power: Some(2),
        toughness: Some(3),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Menace),
            // CR 510.3a / CR 603.2c: "Whenever one or more creatures you control deal combat
            // damage to a player, create a Treasure token." — batch trigger (fires once).
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenOneOrMoreCreaturesYouControlDealCombatDamageToPlayer { filter: None },
                effect: Effect::CreateToken {
                    spec: treasure_token_spec(1),
                },
                intervening_if: None,
                targets: vec![],

                modes: None,
                trigger_zone: None,
            },
            // TODO: "Sacrifice a Treasure: Exile the top card of your library. You may play that card this turn." — impulse draw not expressible.
        ],
        ..Default::default()
    }
}
