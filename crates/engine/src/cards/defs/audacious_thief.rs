// 60. Audacious Thief — {2B}, Creature — Human Rogue 2/2;
// Whenever this creature attacks, you draw a card and you lose 1 life.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("audacious-thief"),
        name: "Audacious Thief".to_string(),
        mana_cost: Some(ManaCost { generic: 2, black: 1, ..Default::default() }),
        types: creature_types(&["Human", "Rogue"]),
        oracle_text: "Whenever Audacious Thief attacks, you draw a card and you lose 1 life."
            .to_string(),
        power: Some(2),
        toughness: Some(2),
        abilities: vec![AbilityDefinition::Triggered {
            trigger_condition: TriggerCondition::WhenAttacks,
            effect: Effect::Sequence(vec![
                Effect::DrawCards {
                    player: PlayerTarget::Controller,
                    count: EffectAmount::Fixed(1),
                },
                Effect::LoseLife {
                    player: PlayerTarget::Controller,
                    amount: EffectAmount::Fixed(1),
                },
            ]),
            intervening_if: None,
            targets: vec![],
        }],
        color_indicator: None,
        back_face: None,
        spell_cost_modifiers: vec![],
        self_cost_reduction: None,
        starting_loyalty: None,
        adventure_face: None,
        meld_pair: None,
        spell_additional_costs: vec![],
        activated_ability_cost_reductions: vec![],
    }
}
