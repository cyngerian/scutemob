// City of Brass — Land
// "Whenever City of Brass becomes tapped, it deals 1 damage to you."
// "{T}: Add one mana of any color."
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("city-of-brass"),
        name: "City of Brass".to_string(),
        mana_cost: None,
        types: types(&[CardType::Land]),
        oracle_text: "Whenever City of Brass becomes tapped, it deals 1 damage to you.\n{T}: Add one mana of any color.".to_string(),
        abilities: vec![
            // Triggered: whenever this becomes tapped (any source), deal 1 damage to controller.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenSelfBecomesTapped,
                effect: Effect::DealDamage {
                    target: EffectTarget::Controller,
                    amount: EffectAmount::Fixed(1),
                },
                intervening_if: None,
            },
            // Mana ability: {T}: Add one mana of any color.
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddManaAnyColor { player: PlayerTarget::Controller },
                timing_restriction: None,
            },
        ],
        ..Default::default()
    }
}
