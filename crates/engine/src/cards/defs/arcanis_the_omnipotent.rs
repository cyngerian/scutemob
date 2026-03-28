// Arcanis the Omnipotent — {3}{U}{U}{U} Legendary Creature — Wizard 3/4
// {T}: Draw three cards.
// {2}{U}{U}: Return Arcanis to its owner's hand.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("arcanis-the-omnipotent"),
        name: "Arcanis the Omnipotent".to_string(),
        mana_cost: Some(ManaCost { generic: 3, blue: 3, ..Default::default() }),
        types: full_types(&[SuperType::Legendary], &[CardType::Creature], &["Wizard"]),
        oracle_text: "{T}: Draw three cards.\n{2}{U}{U}: Return Arcanis to its owner's hand.".to_string(),
        power: Some(3),
        toughness: Some(4),
        abilities: vec![
            // {T}: Draw three cards.
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::DrawCards {
                    player: PlayerTarget::Controller,
                    count: EffectAmount::Fixed(3),
                },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
                activation_zone: None,
            },
            // {2}{U}{U}: Return Arcanis to its owner's hand.
            // NOTE: Uses PlayerTarget::Controller as proxy for "owner" — no PlayerTarget::Owner
            // exists. Wrong under Bribery/steal effects (systemic DSL gap, not Arcanis-specific).
            AbilityDefinition::Activated {
                cost: Cost::Mana(ManaCost { generic: 2, blue: 2, ..Default::default() }),
                effect: Effect::MoveZone {
                    target: EffectTarget::Source,
                    to: ZoneTarget::Hand { owner: PlayerTarget::Controller },
                    controller_override: None,
                },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
                activation_zone: None,
            },
        ],
        ..Default::default()
    }
}
