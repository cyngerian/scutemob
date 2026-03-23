// Ragavan, Nimble Pilferer — {R}, Legendary Creature — Monkey Pirate 2/1
// Whenever Ragavan deals combat damage to a player, create a Treasure token and exile
// the top card of that player's library. Until end of turn, you may cast that card.
// Dash {1}{R}
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("ragavan-nimble-pilferer"),
        name: "Ragavan, Nimble Pilferer".to_string(),
        mana_cost: Some(ManaCost { red: 1, ..Default::default() }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Creature],
            &["Monkey", "Pirate"],
        ),
        oracle_text: "Whenever Ragavan, Nimble Pilferer deals combat damage to a player, create a Treasure token and exile the top card of that player's library. Until end of turn, you may cast that card.\nDash {1}{R} (You may cast this spell for its dash cost. If you do, it gains haste, and it's returned from the battlefield to its owner's hand at the beginning of the next end step.)".to_string(),
        power: Some(2),
        toughness: Some(1),
        abilities: vec![
            // Combat damage: create Treasure + impulse draw
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenDealsCombatDamageToPlayer,
                effect: Effect::CreateToken { spec: treasure_token_spec(1) },
                // TODO: Exile top card + impulse draw not expressible.
                intervening_if: None,
                targets: vec![],
            },
            AbilityDefinition::Keyword(KeywordAbility::Dash),
            AbilityDefinition::AltCastAbility {
                kind: AltCostKind::Dash,
                cost: ManaCost { generic: 1, red: 1, ..Default::default() },
                details: None,
            },
        ],
        ..Default::default()
    }
}
