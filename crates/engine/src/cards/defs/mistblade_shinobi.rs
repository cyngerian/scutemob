// Mistblade Shinobi — {2}{U}, Creature — Human Ninja 1/1
// Ninjutsu {U}
// Whenever this creature deals combat damage to a player, you may return target creature
// that player controls to its owner's hand.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("mistblade-shinobi"),
        name: "Mistblade Shinobi".to_string(),
        mana_cost: Some(ManaCost { generic: 2, blue: 1, ..Default::default() }),
        types: creature_types(&["Human", "Ninja"]),
        oracle_text: "Ninjutsu {U} ({U}, Return an unblocked attacker you control to hand: Put this card onto the battlefield from your hand tapped and attacking.)\nWhenever this creature deals combat damage to a player, you may return target creature that player controls to its owner's hand.".to_string(),
        power: Some(1),
        toughness: Some(1),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Ninjutsu),
            AbilityDefinition::Ninjutsu {
                cost: ManaCost { blue: 1, ..Default::default() },
            },
            // TODO: "you may return target creature that player controls to its owner's hand"
            // — requires targeting a creature controlled by the damaged player + MoveZone.
            // "that player controls" filter is not expressible with current TargetRequirement.
        ],
        ..Default::default()
    }
}
