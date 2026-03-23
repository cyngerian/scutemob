// Up the Beanstalk — {1}{G}, Enchantment
// When this enchantment enters and whenever you cast a spell with mana value 5 or greater,
// draw a card.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("up-the-beanstalk"),
        name: "Up the Beanstalk".to_string(),
        mana_cost: Some(ManaCost { generic: 1, green: 1, ..Default::default() }),
        types: types(&[CardType::Enchantment]),
        oracle_text: "When this enchantment enters and whenever you cast a spell with mana value 5 or greater, draw a card.".to_string(),
        abilities: vec![
            // ETB: draw a card
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenEntersBattlefield,
                effect: Effect::DrawCards {
                    player: PlayerTarget::Controller,
                    count: EffectAmount::Fixed(1),
                },
                intervening_if: None,
                targets: vec![],
            },
            // TODO: "Whenever you cast a spell with mana value 5 or greater" — WheneverYouCastSpell
            //   lacks mana value filter. Overbroad trigger removed to avoid wrong game state.
        ],
        ..Default::default()
    }
}
