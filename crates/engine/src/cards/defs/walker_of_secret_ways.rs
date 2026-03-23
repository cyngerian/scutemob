// Walker of Secret Ways — {2}{U}, Creature — Human Ninja 1/2
// Ninjutsu {1}{U}
// Whenever this creature deals combat damage to a player, look at that player's hand.
// {1}{U}: Return target Ninja you control to its owner's hand. Activate only during your turn.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("walker-of-secret-ways"),
        name: "Walker of Secret Ways".to_string(),
        mana_cost: Some(ManaCost { generic: 2, blue: 1, ..Default::default() }),
        types: creature_types(&["Human", "Ninja"]),
        oracle_text: "Ninjutsu {1}{U} ({1}{U}, Return an unblocked attacker you control to hand: Put this card onto the battlefield from your hand tapped and attacking.)\nWhenever this creature deals combat damage to a player, look at that player's hand.\n{1}{U}: Return target Ninja you control to its owner's hand. Activate only during your turn.".to_string(),
        power: Some(1),
        toughness: Some(2),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Ninjutsu),
            AbilityDefinition::Ninjutsu {
                cost: ManaCost { generic: 1, blue: 1, ..Default::default() },
            },
            // TODO: "look at that player's hand" — hidden information reveal, no DSL support.
            // TODO: "{1}{U}: Return target Ninja you control to its owner's hand" — requires
            // subtype-filtered targeting (Ninja) + MoveZone to hand + "activate only during your turn".
        ],
        ..Default::default()
    }
}
