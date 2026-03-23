// Saskia the Unyielding — {B}{R}{G}{W}, Legendary Creature — Human Soldier 3/4
// Vigilance, haste
// As Saskia enters, choose a player.
// Whenever a creature you control deals combat damage to a player, it deals that much
// damage to the chosen player.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("saskia-the-unyielding"),
        name: "Saskia the Unyielding".to_string(),
        mana_cost: Some(ManaCost { black: 1, red: 1, green: 1, white: 1, ..Default::default() }),
        types: full_types(&[SuperType::Legendary], &[CardType::Creature], &["Human", "Soldier"]),
        oracle_text: "Vigilance, haste\nAs Saskia enters, choose a player.\nWhenever a creature you control deals combat damage to a player, it deals that much damage to the chosen player.".to_string(),
        power: Some(3),
        toughness: Some(4),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Vigilance),
            AbilityDefinition::Keyword(KeywordAbility::Haste),
            // TODO: DSL gap — "As ~ enters, choose a player" is a replacement/choice effect
            // on ETB with persistent state (the chosen player). No ChoosePlayer ETB mechanic
            // exists in the DSL.
            // TODO: DSL gap — "Whenever a creature you control deals combat damage to a player,
            // it deals that much damage to the chosen player" requires a combat damage trigger
            // keyed on a previously chosen player target. No such trigger condition exists.
        ],
        ..Default::default()
    }
}
