// Ripjaw Raptor — {2}{G}{G}, Creature — Dinosaur 4/5; Enrage (ability word):
// Whenever this creature is dealt damage, draw a card.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("ripjaw-raptor"),
        name: "Ripjaw Raptor".to_string(),
        mana_cost: Some(ManaCost { generic: 2, green: 2, ..Default::default() }),
        types: creature_types(&["Dinosaur"]),
        oracle_text: "Enrage — Whenever this creature is dealt damage, draw a card."
            .to_string(),
        power: Some(4),
        toughness: Some(5),
        abilities: vec![
            // Enrage is an ability word, not a KeywordAbility variant.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenDealtDamage,
                effect: Effect::DrawCards {
                    player: PlayerTarget::Controller,
                    count: EffectAmount::Fixed(1),
                },
                intervening_if: None,
                targets: vec![],
            },
        ],
        color_indicator: None,
        back_face: None,
        spell_cost_modifiers: vec![],
        self_cost_reduction: None,
        starting_loyalty: None,
        adventure_face: None,
        meld_pair: None,
    }
}
