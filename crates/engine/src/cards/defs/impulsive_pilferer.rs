// Impulsive Pilferer — {R}, Creature — Goblin Pirate 1/1
// When this creature dies, create a Treasure token.
// Encore {3}{R}
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("impulsive-pilferer"),
        name: "Impulsive Pilferer".to_string(),
        mana_cost: Some(ManaCost { red: 1, ..Default::default() }),
        types: creature_types(&["Goblin", "Pirate"]),
        oracle_text: "When this creature dies, create a Treasure token. (It's an artifact with \"{T}, Sacrifice this token: Add one mana of any color.\")\nEncore {3}{R} ({3}{R}, Exile this card from your graveyard: For each opponent, create a token copy that attacks that opponent this turn if able. They gain haste. Sacrifice them at the beginning of the next end step. Activate only as a sorcery.)".to_string(),
        power: Some(1),
        toughness: Some(1),
        abilities: vec![
            // When this creature dies, create a Treasure token.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenDies,
                effect: Effect::CreateToken {
                    spec: treasure_token_spec(1),
                },
                intervening_if: None,
                targets: vec![],

                modes: None,
                trigger_zone: None,
            },
            // Encore {3}{R}
            AbilityDefinition::Keyword(KeywordAbility::Encore),
            AbilityDefinition::AltCastAbility {
                kind: AltCostKind::Encore,
                cost: ManaCost { generic: 3, red: 1, ..Default::default() },
                details: None,
            },
        ],
        ..Default::default()
    }
}
