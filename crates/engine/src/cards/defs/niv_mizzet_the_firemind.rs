// Niv-Mizzet, the Firemind — {2}{U}{U}{R}{R} Legendary Creature — Dragon Wizard 4/4
// Flying
// Whenever you draw a card, Niv-Mizzet deals 1 damage to any target.
// {T}: Draw a card.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("niv-mizzet-the-firemind"),
        name: "Niv-Mizzet, the Firemind".to_string(),
        mana_cost: Some(ManaCost { generic: 2, blue: 2, red: 2, ..Default::default() }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Creature],
            &["Dragon", "Wizard"],
        ),
        oracle_text: "Flying\nWhenever you draw a card, Niv-Mizzet, the Firemind deals 1 damage to any target.\n{T}: Draw a card.".to_string(),
        power: Some(4),
        toughness: Some(4),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            // TODO: "Whenever you draw a card" trigger — TriggerEvent::WheneverYouDrawCard
            // not in DSL. Needs draw-event trigger.
            // {T}: Draw a card.
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::DrawCards {
                    player: PlayerTarget::Controller,
                    count: EffectAmount::Fixed(1),
                },
                targets: vec![],
                timing_restriction: None,
                activation_condition: None,
            },
        ],
        ..Default::default()
    }
}
